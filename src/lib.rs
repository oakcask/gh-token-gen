use base64ct::{Base64UrlUnpadded, Encoding};
use http::Uri;
use log::error;
mod sign;
use serde::{Deserialize, Serialize};
use sign::sign_sha256;
use wasm_actions::{
    derive::{wasm_action, ActionInput, ActionOutput},
    prelude::{add_mask, derive::Action},
};
use wasm_actions_core::error::Error;

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
            endpoint: Uri::try_from(input.endpoint).map_err(Error::new)?,
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
            endpoint: Uri::try_from(input.endpoint).map_err(Error::new)?,
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
    #[input(env = "GITHUB_API_URL", default = "https://api.github.com")]
    endpoint: String,
    #[input(env = "GITHUB_REPOSITORY")]
    repo: String,
}

#[derive(ActionOutput, serde::Serialize, serde::Deserialize)]
struct Output {
    #[output(name = "token", description = "Generated Token")]
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
    endpoint: Uri,
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
        let api = Uri::builder()
            .scheme(
                self.endpoint
                    .scheme()
                    .ok_or_else(|| Error::from("endpoint (GITHUB_API_URL) must contain scheme"))?
                    .as_str(),
            )
            .authority(
                self.endpoint
                    .authority()
                    .ok_or_else(|| Error::from("endpoint (GITHUB_API_URL) must contain authority"))?
                    .as_str(),
            )
            .path_and_query(path)
            .build()
            .map_err(Error::new)?;

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
        let api = Uri::builder()
            .scheme(
                self.endpoint
                    .scheme()
                    .expect("endpoint (GITHUB_API_URL) must contain scheme")
                    .as_str(),
            )
            .authority(
                self.endpoint
                    .authority()
                    .expect("endpoint (GITHUB_API_URL) must contain authority")
                    .as_str(),
            )
            .path_and_query(path)
            .build()
            .map_err(Error::new)?;

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
    endpoint: Uri,
    token: String,
    installation_id: u64,
    client: reqwest::Client,
}

impl RemoveAccessTokenRequest {
    async fn execute(self) -> Result<(), Error> {
        let path = format!("/app/installations/{}/access_tokens", self.installation_id);
        let api = Uri::builder()
            .scheme(
                self.endpoint
                    .scheme()
                    .expect("endpoint (GITHUB_API_URL) must contain scheme")
                    .as_str(),
            )
            .authority(
                self.endpoint
                    .authority()
                    .expect("endpoint (GITHUB_API_URL) must contain authority")
                    .as_str(),
            )
            .path_and_query(path)
            .build()
            .map_err(Error::new)?;

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
