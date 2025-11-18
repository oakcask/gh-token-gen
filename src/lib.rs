use base64ct::{Base64UrlUnpadded, Encoding};
use builder::Action;
use derive::{ActionInput, ActionOutput, wasm_action};
use http::Uri;
use log::error;
mod sign;
use serde::{Deserialize, Serialize};
use sign::sign_sha256;
use wasm_actions::{add_mask, console, env, get_input, get_state, save_state, set_output};
use wasm_actions_core::error::Error;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::prelude::JsError;

#[wasm_action(
    name = "gh-token-gen",
    description = "generate GitHub application token"
)]
struct GhTokenGen;

#[wasm_bindgen]
pub async fn main() -> Result<(), JsError> {
    console::init().map_err(|e| Error::from(e.to_string()))?;

    let cli = Cli::try_from_env()?;

    Ok(cli.run().await?)
}

#[derive(ActionInput)]
struct Input {
    #[input(
        name = "app-id",
        required = true,
        description = "GitHub Application ID (or Client ID)")]
    app_id: String,
    #[input(
        name = "private-key",
        required = true,
        description = "The application's PEM-encoded private key")]
    private_key: String,
    #[input(env = "GITHUB_API_URL", default = "https://api.github.com")]
    endpoint: String,
    #[input(env = "GITHUB_REPOSITORY")]
    repo: String,   
}
// impl ActionInput for Input {
//   fn parse() -> Result<Self, Error> {
//     Ok(Self {
//       #field1 : get_input!(#name).ok_or_else(|| Error::from("{} missing", #name))?.try_into().map_err(|e| Error::from(format!("error while parsing {}: {}", #name, e)))?,
//       #field2 : env::var(#env).unwrap_or_else(|| Error::from("${} missing", #env))?.try_into().map_err(|e| Error::new(e))?,
//     })
//   }
// }

#[derive(ActionOutput)]
struct Output {
    #[output(
        name = "token",
        description = "Generated Token")]
    token: String
}

impl Action<Input, Output> for GhTokenGen {
    async fn main(input: Input) -> Result<Output, Error> {
        todo!()
    }
}


#[derive(Debug)]
enum Run {
    Main,
    Post { access_token: AccessToken },
}

#[derive(Debug)]
struct Cli {
    app_id: String,
    private_key: String,
    endpoint: String,
    repo: String,
    run: Run,
}

impl Cli {
    fn try_from_env() -> Result<Self, Error> {
        let app_id = get_input!("app-id").ok_or_else(|| Error::from("app-id missing"))?;
        let private_key =
            get_input!("private-key").ok_or_else(|| Error::from("private-key missing"))?;
        let endpoint =
            env::var("GITHUB_API_URL").unwrap_or_else(|| String::from("https://api.github.com"));
        let repo = env::var("GITHUB_REPOSITORY")
            .ok_or_else(|| Error::from("GITHUB_REPOSITORY missing"))?;

        Ok(Self {
            app_id,
            private_key,
            endpoint,
            repo,
            run: Run::from_env()?,
        })
    }

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

    async fn run(self) -> Result<(), Error> {
        match self.run {
            Run::Main => {
                let authorization_header = JwtBuilder {
                    payload: Self::create_payload(self.app_id)?,
                    pkey: self.private_key,
                }
                .build_authorization_header()
                .await?;
                add_mask(&authorization_header);

                let access_token = AccessTokenBuilder {
                    endpoint: Uri::try_from(self.endpoint).map_err(Error::new)?,
                    repo: self.repo,
                    authorization_header,
                    client: reqwest::Client::new(),
                }
                .build()
                .await?;

                set_output("token", &access_token.token).await?;
                add_mask(&access_token.token);
                let state = serde_json::to_string(&access_token).map_err(Error::new)?;
                save_state("access_token", &state).await
            }
            Run::Post { access_token } => {
                RemoveAccessTokenRequest {
                    endpoint: Uri::try_from(self.endpoint).map_err(Error::new)?,
                    installation_id: access_token.installation_id,
                    token: access_token.token,
                    client: reqwest::Client::new(),
                }
                .execute()
                .await
            }
        }
    }
}

impl Run {
    fn from_env() -> Result<Self, Error> {
        if let Some(access_token) = get_state!("access_token") {
            let access_token: AccessToken =
                serde_json::from_str(&access_token).map_err(Error::new)?;
            Ok(Self::Post { access_token })
        } else {
            Ok(Self::Main)
        }
    }
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
