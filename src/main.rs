use std::{
    error::Error,
    fs::File,
    io::Write as _,
    time::{self, Duration},
};

use base64ct::{Base64UrlUnpadded, Encoding};
use clap::Parser;
use http::Uri;
use log::{debug, error, info};
use openssl::{
    hash::MessageDigest,
    pkey::{PKey, Private},
    sign::Signer,
};
use serde::{Deserialize, Serialize};

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    Cli::parse().run()
}

#[derive(Parser, Debug)]
#[command(
    about = "generate GitHub token for actions",
    long_about = None
)]
struct Cli {
    #[arg(long, env = "INPUT_APP_ID", help = "GitHub Application ID")]
    app_id: String,
    #[arg(
        long,
        env = "INPUT_PRIVATE_KEY",
        help = "PEM-encoded private key to sign a JWT"
    )]
    private_key: String,
    #[arg(
        long,
        env = "INPUT_EXPIRE",
        default_value = "120",
        help = "Set ttl for generated token in specified seconds. Limited to maximum 600 seconds."
    )]
    expire: u32,
    #[arg(
        long,
        env = "GITHUB_API_URL",
        default_value = "https://api.github.com",
        help = "URL points to where GitHub API served"
    )]
    endpoint: String,
    #[arg(
        long,
        env = "GITHUB_REPOSITORY",
        help = "GitHub repository name formed `<owner>/<name>`"
    )]
    repo: String,
    #[arg(
        short = 'o',
        long = "gh-out",
        help = "Write result into ${GITHUB_OUTPUT} with specified name"
    )]
    gh_out: Option<String>,
}

impl Cli {
    fn create_payload(app_id: String, expire: u32) -> Payload {
        let now = unix_now();
        let expire_in = if expire > 600 { 600 } else { expire };

        Payload {
            iss: app_id,
            // 60 seconds in the past to allow for clock drift
            // https://docs.github.com/en/apps/creating-github-apps/authenticating-with-a-github-app/generating-a-json-web-token-jwt-for-a-github-app#example-using-ruby-to-generate-a-jwt
            iat: (now - Duration::from_secs(60)).as_secs(),
            exp: (now + Duration::from_secs(expire_in as u64)).as_secs(),
        }
    }

    fn run(self) -> Result<(), Box<dyn Error>> {
        let authorization_header = JwtBuilder {
            payload: Self::create_payload(self.app_id, self.expire),
            pkey: PKey::private_key_from_pem(self.private_key.as_bytes())?,
        }
        .build_authorization_header()?;
        add_mask(&authorization_header, self.gh_out.is_some());

        let access_token = AccessTokenBuilder {
            endpoint: Uri::try_from(self.endpoint)?,
            repo: self.repo,
            authorization_header,
            client: reqwest::blocking::Client::new(),
            gh_out_enabled: self.gh_out.is_some(),
        }
        .build()?;

        print_result(access_token, self.gh_out)
    }
}

#[derive(Serialize)]
struct Payload {
    iss: String,
    iat: u64,
    exp: u64,
}

struct JwtBuilder {
    payload: Payload,
    pkey: PKey<Private>,
}

impl JwtBuilder {
    fn build_authorization_header(self) -> Result<String, Box<dyn Error>> {
        const HEADER: &[u8] = r#"{"alg":"RS256","typ":"JWT"}"#.as_bytes();
        let header = encode_base64_url(HEADER);
        let payload = serde_json::to_string(&self.payload)?;
        debug!("using payload: {:?}", payload);

        let payload = encode_base64_url(payload.as_bytes());
        let sig = format!("{}.{}", header, payload);
        let sig = sign_sha256(sig.as_bytes(), &self.pkey)?;
        let sig = encode_base64_url(&sig);
        Ok(format!("Bearer {}.{}.{}", header, payload, sig))
    }
}

struct AccessTokenBuilder {
    endpoint: Uri,
    repo: String,
    authorization_header: String,
    client: reqwest::blocking::Client,
    gh_out_enabled: bool,
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
    fn get_installation_id(&self) -> Result<u64, Box<dyn Error>> {
        let path = format!("/repos/{}/installation", self.repo);
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
            .build()?;

        let res = self
            .client
            .get(api.to_string())
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", USER_AGENT)
            .header("X-GitHub-Api-Version", "2022-11-28")
            .header("Authorization", self.authorization_header.clone())
            .send()?;

        if let Err(e) = res.error_for_status_ref() {
            error!("{:?}", res.bytes()?);
            Err(Box::new(e))
        } else {
            let res: InstallationResponse = res.json()?;
            info!("found installation id: {}", res.id);
            Ok(res.id)
        }
    }

    fn build(self) -> Result<String, Box<dyn Error>> {
        let reponame = self.repo.split('/').nth(1).unwrap();
        let path = format!(
            "/app/installations/{}/access_tokens",
            self.get_installation_id()?
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
            .build()?;

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
            .send()?;

        if let Err(e) = res.error_for_status_ref() {
            error!("{:?}", res.bytes()?);
            Err(Box::new(e))
        } else {
            let res: AccessTokenResponse = res.json()?;
            add_mask(&res.token, self.gh_out_enabled);
            Ok(res.token)
        }
    }
}

fn print_result(token: String, gh_out: Option<String>) -> Result<(), Box<dyn Error>> {
    if let Some(out) = gh_out {
        let path = std::env::var("GITHUB_OUTPUT")
            .expect("with `--gh-out` option, $GITHUB_OUTPUT must be set");
        let mut f = File::options().append(true).create(true).open(path)?;
        writeln!(&mut f, "{}={}", out, token)?;
    } else {
        println!("{}", token);
    }

    Ok(())
}

fn add_mask(value: &str, gh_out_enabled: bool) {
    if gh_out_enabled {
        println!("::add-mask::{}", value)
    }
}

fn unix_now() -> Duration {
    // Since we live after 1970, this always succeeds.
    time::SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .unwrap()
}

fn encode_base64_url(src: &[u8]) -> String {
    Base64UrlUnpadded::encode_string(src)
}

fn sign_sha256(buf: &[u8], pkey: &PKey<Private>) -> Result<Vec<u8>, openssl::error::ErrorStack> {
    let mut signer = Signer::new(MessageDigest::sha256(), pkey)?;
    signer.sign_oneshot_to_vec(buf)
}
