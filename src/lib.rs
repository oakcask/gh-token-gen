use base64ct::{Base64UrlUnpadded, Encoding};
use error::Error;
use http::Uri;
use log::{error, info};
mod env;
mod error;
mod logging;
mod sign;
use logging::{add_mask, set_output};
use serde::{Deserialize, Serialize};
use sign::sign_sha256;
use web_sys::wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub async fn start() -> Result<(), JsError> {
    logging::init();

    let cli = Cli::try_from_env()?;

    Ok(cli.run().await?)
}

#[derive(Debug)]
struct Cli {
    app_id: String,
    private_key: String,
    expire: u32,
    endpoint: String,
    repo: String,
}

use macros::{input_var, input_var_underscore};

macro_rules! get_input {
    ($name:expr) => {
        if let Some(value) = env::var(input_var!($name)) {
            Some(value)
        } else if let Some(value) = env::var(input_var_underscore!($name)) {
            Some(value)
        } else {
            None
        }
    };
}

impl Cli {
    fn try_from_env() -> Result<Self, Error> {
        let app_id = get_input!("app-id").ok_or_else(|| Error::from("app-id missing"))?;
        let private_key =
            get_input!("private-key").ok_or_else(|| Error::from("private-key missing"))?;
        let expire = get_input!("expire-in")
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or_else(|| "120")
            .parse::<u32>()
            .map_err(|_| Error::from("expire-in must be a integer"))?;
        let endpoint =
            env::var("GITHUB_API_URL").unwrap_or_else(|| String::from("https://api.github.com"));
        let repo = env::var("GITHUB_REPOSITORY")
            .ok_or_else(|| Error::from("GITHUB_REPOSITORY missing"))?;

        Ok(Self {
            app_id,
            private_key,
            expire,
            endpoint,
            repo,
        })
    }

    fn create_payload(app_id: String, expire: u32) -> Result<Payload, Error> {
        let now = unix_now();
        let expire_in = if expire > 600 { 600 } else { expire as i64 };

        Ok(Payload {
            iss: app_id,
            // 60 seconds in the past to allow for clock drift
            // https://docs.github.com/en/apps/creating-github-apps/authenticating-with-a-github-app/generating-a-json-web-token-jwt-for-a-github-app#example-using-ruby-to-generate-a-jwt
            iat: now - 60,
            exp: now + expire_in,
        })
    }

    async fn run(self) -> Result<(), Error> {
        let authorization_header = JwtBuilder {
            payload: Self::create_payload(self.app_id, self.expire)?,
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

        set_output("token", &access_token)
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
        let sig = format!("{}.{}", header, payload);
        let sig = sign_sha256(sig.as_bytes(), &self.pkey).await?;
        let sig = encode_base64_url(&sig);
        Ok(format!("Bearer {}.{}.{}", header, payload, sig))
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
            info!("found installation id: {}", res.id);
            Ok(res.id)
        }
    }

    async fn build(self) -> Result<String, Error> {
        let reponame = self.repo.split('/').nth(1).unwrap();
        let path = format!(
            "/app/installations/{}/access_tokens",
            self.get_installation_id().await?
        );
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
            Ok(res.token)
        }
    }
}

fn unix_now() -> i64 {
    chrono::Utc::now().timestamp()
}

fn encode_base64_url(src: &[u8]) -> String {
    Base64UrlUnpadded::encode_string(src)
}
