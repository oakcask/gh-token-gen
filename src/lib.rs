use base64ct::{Base64UrlUnpadded, Encoding};
use http::{uri::Authority, Uri};
use log::error;
mod sign;
use serde::{Deserialize, Serialize};
use sign::sign_sha256;
use wasm_actions::{
    derive::{wasm_action, ActionInput, ActionOutput},
    prelude::{add_mask, derive::Action, Error},
};

#[wasm_action(
    name = "gh-token-gen",
    description = "generate GitHub application token"
)]
struct GhTokenGen;

impl GhTokenGen {
    fn create_payload(app_id: String) -> Result<Payload, Error> {
        let now = unix_now();

        Ok(Payload {
            iss: app_id,
            // 60 seconds in the past to allow for clock drift
            // https://docs.github.com/en/apps/creating-github-apps/authenticating-with-a-github-app/generating-a-json-web-token-jwt-for-a-github-app#example-using-ruby-to-generate-a-jwt
            iat: now - 60,
            exp: now + 60,
        })
    }
}

impl Action<Input, Output> for GhTokenGen {
    async fn main(input: Input) -> Result<Output, Error> {
        let authorization_header = JwtBuilder {
            payload: Self::create_payload(input.app_id)?,
            pkey: input.private_key,
        }
        .build_authorization_header()
        .await?;
        add_mask(&authorization_header);

        let access_token = AccessTokenBuilder {
            endpoint: ApiEndpoint::from_inputs(&input.github_api_url, &input.endpoint)?,
            repo: input.repo,
            authorization_header,
            client: reqwest::Client::new(),
        }
        .build()
        .await?;
        add_mask(&access_token.token);
        Ok(Output {
            token: access_token.token,
            installation_id: access_token.installation_id,
        })
    }

    async fn post(input: Input, state: Output) -> Result<(), Error> {
        RemoveAccessTokenRequest {
            endpoint: ApiEndpoint::from_inputs(&input.github_api_url, &input.endpoint)?,
            installation_id: state.installation_id,
            token: state.token,
            client: reqwest::Client::new(),
        }
        .execute()
        .await
    }
}

#[derive(ActionInput)]
struct Input {
    #[input(
        name = "app-id",
        required = true,
        description = "GitHub Application ID (or Client ID)"
    )]
    app_id: String,
    #[input(
        name = "private-key",
        required = true,
        description = "The application's PEM-encoded private key"
    )]
    private_key: String,
    #[input(
        name = "github-api-url",
        default = "https://api.github.com",
        description = "GitHub API URL; override this for GHES"
    )]
    github_api_url: String,
    #[input(
        name = "endpoint",
        default = "",
        description = "Deprecated alias for github-api-url"
    )]
    endpoint: String,
    #[input(env = "GITHUB_REPOSITORY")]
    repo: String,
}

#[derive(Clone)]
struct ApiEndpoint {
    scheme: String,
    authority: Authority,
    base_path: String,
}

impl ApiEndpoint {
    fn from_inputs(github_api_url: &str, endpoint: &str) -> Result<Self, Error> {
        let value = if endpoint.trim().is_empty() {
            github_api_url
        } else {
            endpoint
        };
        Self::parse(value)
    }

    fn parse(value: &str) -> Result<Self, Error> {
        let uri = Uri::try_from(value.trim()).map_err(Error::new)?;
        let scheme = uri
            .scheme()
            .ok_or_else(|| Error::from("github-api-url must contain scheme"))?
            .as_str()
            .to_string();
        let authority = uri
            .authority()
            .ok_or_else(|| Error::from("github-api-url must contain authority"))?
            .clone();
        let base_path = uri
            .path_and_query()
            .map(|path| path.path().trim_end_matches('/').to_string())
            .filter(|path| !path.is_empty() && path != "/")
            .unwrap_or_default();

        Ok(Self {
            scheme,
            authority,
            base_path,
        })
    }

    fn uri(&self, path: &str) -> Result<Uri, Error> {
        let path = format!("{}{}", self.base_path, path);
        Uri::builder()
            .scheme(self.scheme.as_str())
            .authority(self.authority.as_str())
            .path_and_query(path)
            .build()
            .map_err(Error::new)
    }
}

#[derive(ActionOutput, serde::Serialize, serde::Deserialize)]
struct Output {
    #[output(name = "token", description = "Generated token")]
    token: String,
    installation_id: u64,
}

#[derive(Serialize)]
struct Payload {
    iss: String,
    iat: i64,
    exp: i64,
}

struct JwtBuilder {
    payload: Payload,
    pkey: String,
}

impl JwtBuilder {
    async fn build_authorization_header(self) -> Result<String, Error> {
        const HEADER: &[u8] = r#"{"alg":"RS256","typ":"JWT"}"#.as_bytes();
        let header = encode_base64_url(HEADER);
        let payload = serde_json::to_string(&self.payload).map_err(Error::new)?;
        let payload = encode_base64_url(payload.as_bytes());
        let sig = format!("{header}.{payload}");
        let sig = sign_sha256(sig.as_bytes(), &self.pkey).await?;
        let sig = encode_base64_url(&sig);
        Ok(format!("Bearer {header}.{payload}.{sig}"))
    }
}

struct AccessTokenBuilder {
    endpoint: ApiEndpoint,
    repo: String,
    authorization_header: String,
    client: reqwest::Client,
}

#[derive(Deserialize)]
struct InstallationResponse {
    id: u64,
}

#[derive(Serialize)]
struct AccessTokenRequest<'a> {
    repositories: [&'a str; 1],
}

#[derive(Deserialize)]
struct AccessTokenResponse {
    token: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AccessToken {
    installation_id: u64,
    token: String,
}

const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

impl AccessTokenBuilder {
    async fn get_installation_id(&self) -> Result<u64, Error> {
        let path = format!("/repos/{}/installation", self.repo);
        let api = self.endpoint.uri(&path)?;

        let res = self
            .client
            .get(api.to_string())
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", USER_AGENT)
            .header("X-GitHub-Api-Version", "2022-11-28")
            .header("Authorization", self.authorization_header.clone())
            .send()
            .await
            .map_err(|e| Error::from(e.to_string()))?;

        if let Err(e) = res.error_for_status_ref() {
            error!("{:?}", res.bytes().await.map_err(Error::new)?);
            Err(Error::new(e))
        } else {
            let res: InstallationResponse = res.json().await.map_err(Error::new)?;
            Ok(res.id)
        }
    }

    async fn build(self) -> Result<AccessToken, Error> {
        let reponame = self.repo.split('/').nth(1).unwrap();
        let installation_id = self.get_installation_id().await?;
        let path = format!("/app/installations/{}/access_tokens", installation_id);
        let api = self.endpoint.uri(&path)?;

        let body = AccessTokenRequest {
            repositories: [reponame],
        };
        let res = self
            .client
            .post(api.to_string())
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", USER_AGENT)
            .header("X-GitHub-Api-Version", "2022-11-28")
            .header("Authorization", self.authorization_header)
            .json(&body)
            .send()
            .await
            .map_err(Error::new)?;

        if let Err(e) = res.error_for_status_ref() {
            error!("{:?}", res.bytes().await.map_err(Error::new)?);
            Err(Error::from(e.to_string()))
        } else {
            let res: AccessTokenResponse = res.json().await.map_err(Error::new)?;
            add_mask(&res.token);
            Ok(AccessToken {
                installation_id,
                token: res.token,
            })
        }
    }
}

struct RemoveAccessTokenRequest {
    endpoint: ApiEndpoint,
    token: String,
    installation_id: u64,
    client: reqwest::Client,
}

impl RemoveAccessTokenRequest {
    async fn execute(self) -> Result<(), Error> {
        let path = format!("/app/installations/{}/access_tokens", self.installation_id);
        let api = self.endpoint.uri(&path)?;

        let _ = self
            .client
            .delete(api.to_string())
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", USER_AGENT)
            .header("X-GitHub-Api-Version", "2022-11-28")
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .await
            .map_err(Error::new)?;
        Ok(())
    }
}

fn unix_now() -> i64 {
    chrono::Utc::now().timestamp()
}

fn encode_base64_url(src: &[u8]) -> String {
    Base64UrlUnpadded::encode_string(src)
}
