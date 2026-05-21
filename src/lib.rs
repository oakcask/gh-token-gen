use base64ct::{Base64UrlUnpadded, Encoding};
use http::{uri::Authority, Uri};
use log::error;
mod sign;
use serde::{Deserialize, Serialize};
use sign::sign_sha256;
use std::collections::BTreeMap;
use wasm_actions::{
    derive::{wasm_action, ActionInput, ActionOutput},
    prelude::{add_mask, derive::Action, env, Error},
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
        let endpoint = ApiEndpoint::from_inputs(&input.github_api_url, &input.endpoint)?;
        let target = InstallationTarget::resolve(&input)?;
        let authorization_header = JwtBuilder {
            payload: Self::create_payload(input.app_id)?,
            pkey: input.private_key,
        }
        .build_authorization_header()
        .await?;
        add_mask(&authorization_header);

        let access_token = AccessTokenBuilder {
            endpoint,
            target,
            permissions: permissions_from_inputs(),
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
    #[input(
        name = "owner",
        default = "",
        description = "The owner of the GitHub App installation"
    )]
    owner: String,
    #[input(
        name = "repositories",
        default = "",
        description = "Comma or newline-separated list of repositories to grant access to"
    )]
    repositories: String,
    #[input(
        name = "enterprise",
        default = "",
        description = "The slug of the enterprise account where the GitHub App is installed"
    )]
    enterprise: String,
    #[input(env = "GITHUB_REPOSITORY")]
    repo: String,
    #[input(env = "GITHUB_REPOSITORY_OWNER")]
    repo_owner: String,
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
    target: InstallationTarget,
    permissions: Option<BTreeMap<String, String>>,
    authorization_header: String,
    client: reqwest::Client,
}

enum InstallationTarget {
    Enterprise {
        enterprise: String,
    },
    Owner {
        owner: String,
    },
    Repository {
        owner: String,
        repositories: Vec<String>,
    },
}

impl InstallationTarget {
    fn resolve(input: &Input) -> Result<Self, Error> {
        let enterprise = input.enterprise.trim();
        let owner = input.owner.trim();
        let repositories = parse_repositories(&input.repositories);

        if !enterprise.is_empty() {
            if !owner.is_empty() || !repositories.is_empty() {
                return Err(Error::from(
                    "enterprise cannot be used with owner or repositories",
                ));
            }
            return Ok(Self::Enterprise {
                enterprise: enterprise.to_string(),
            });
        }

        if owner.is_empty() && repositories.is_empty() {
            let (owner, repo) = input
                .repo
                .split_once('/')
                .ok_or_else(|| Error::from("GITHUB_REPOSITORY must be '<owner>/<repo>'"))?;
            return Ok(Self::Repository {
                owner: owner.to_string(),
                repositories: vec![repo.to_string()],
            });
        }

        if !owner.is_empty() && repositories.is_empty() {
            return Ok(Self::Owner {
                owner: owner.to_string(),
            });
        }

        let owner = if owner.is_empty() {
            input.repo_owner.trim()
        } else {
            owner
        };
        if owner.is_empty() {
            return Err(Error::from("owner could not be resolved"));
        }

        let repositories = repositories
            .into_iter()
            .map(|repository| parse_repository(owner, &repository))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self::Repository {
            owner: owner.to_string(),
            repositories,
        })
    }

    fn installation_paths(&self) -> Vec<String> {
        match self {
            Self::Enterprise { enterprise } => {
                vec![format!("/enterprises/{enterprise}/installation")]
            }
            Self::Owner { owner } => vec![
                format!("/orgs/{owner}/installation"),
                format!("/users/{owner}/installation"),
            ],
            Self::Repository {
                owner,
                repositories,
            } => vec![format!("/repos/{owner}/{}/installation", repositories[0])],
        }
    }

    fn repository_names(&self) -> Option<Vec<String>> {
        match self {
            Self::Repository { repositories, .. } => Some(repositories.clone()),
            Self::Enterprise { .. } | Self::Owner { .. } => None,
        }
    }
}

fn parse_repositories(input: &str) -> Vec<String> {
    input
        .split([',', '\n'])
        .map(str::trim)
        .filter(|repository| !repository.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn parse_repository(owner: &str, input: &str) -> Result<String, Error> {
    match input.split_once('/') {
        Some((repo_owner, repo)) if repo_owner.is_empty() || repo.is_empty() => Err(Error::from(
            format!("invalid repository '{input}', expected 'repository' or 'owner/repository'"),
        )),
        Some((repo_owner, repo)) if repo_owner.eq_ignore_ascii_case(owner) => Ok(repo.to_string()),
        Some((repo_owner, _)) => Err(Error::from(format!(
            "repository '{input}' includes owner '{repo_owner}', which does not match '{owner}'"
        ))),
        None if !input.is_empty() => Ok(input.to_string()),
        None => Err(Error::from("repository name cannot be empty")),
    }
}

#[derive(Deserialize)]
struct InstallationResponse {
    id: u64,
}

#[derive(Serialize)]
struct AccessTokenRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    repositories: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    permissions: Option<BTreeMap<String, String>>,
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
        let paths = self.target.installation_paths();

        for (index, path) in paths.iter().enumerate() {
            let api = self.endpoint.uri(path)?;
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
                let status = e.status();
                let message = e.to_string();
                let has_next_path = index + 1 < paths.len();
                if status == Some(reqwest::StatusCode::NOT_FOUND) && has_next_path {
                    continue;
                }

                error!("{:?}", res.bytes().await.map_err(Error::new)?);
                return Err(Error::from(message));
            }

            let res: InstallationResponse = res.json().await.map_err(Error::new)?;
            return Ok(res.id);
        }

        Err(Error::from("installation could not be resolved"))
    }

    async fn build(self) -> Result<AccessToken, Error> {
        let installation_id = self.get_installation_id().await?;
        let path = format!("/app/installations/{}/access_tokens", installation_id);
        let api = self.endpoint.uri(&path)?;

        let body = AccessTokenRequest {
            repositories: self.target.repository_names(),
            permissions: self.permissions,
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

fn permissions_from_inputs() -> Option<BTreeMap<String, String>> {
    let permissions = env::vars()
        .filter_map(|(key, value)| {
            let permission = key.strip_prefix("INPUT_PERMISSION-")?;
            if value.trim().is_empty() {
                return None;
            }

            Some((
                permission.to_ascii_lowercase().replace('-', "_"),
                value.trim().to_string(),
            ))
        })
        .collect::<BTreeMap<_, _>>();

    if permissions.is_empty() {
        None
    } else {
        Some(permissions)
    }
}

fn unix_now() -> i64 {
    chrono::Utc::now().timestamp()
}

fn encode_base64_url(src: &[u8]) -> String {
    Base64UrlUnpadded::encode_string(src)
}

#[cfg(all(test, target_arch = "wasm32"))]
mod tests {
    use super::*;
    use wasm_bindgen_test::wasm_bindgen_test;

    fn input() -> Input {
        Input {
            app_id: "client-id".to_string(),
            private_key: "private-key".to_string(),
            github_api_url: "https://api.github.com".to_string(),
            endpoint: String::new(),
            owner: String::new(),
            repositories: String::new(),
            enterprise: String::new(),
            repo: "owner/current".to_string(),
            repo_owner: "owner".to_string(),
        }
    }

    #[wasm_bindgen_test]
    fn parses_default_github_api_url() {
        let endpoint = ApiEndpoint::from_inputs("https://api.github.com", "").unwrap();

        assert_eq!(
            endpoint.uri("/repos/owner/repo/installation").unwrap(),
            "https://api.github.com/repos/owner/repo/installation"
        );
    }

    #[wasm_bindgen_test]
    fn preserves_github_enterprise_api_base_path() {
        let endpoint = ApiEndpoint::from_inputs("https://github.example.com/api/v3/", "").unwrap();

        assert_eq!(
            endpoint
                .uri("/app/installations/123/access_tokens")
                .unwrap(),
            "https://github.example.com/api/v3/app/installations/123/access_tokens"
        );
    }

    #[wasm_bindgen_test]
    fn endpoint_alias_overrides_github_api_url() {
        let endpoint = ApiEndpoint::from_inputs(
            "https://api.github.com",
            "https://github.example.com/api/v3",
        )
        .unwrap();

        assert_eq!(
            endpoint.uri("/installation/token").unwrap(),
            "https://github.example.com/api/v3/installation/token"
        );
    }

    #[wasm_bindgen_test]
    fn rejects_github_api_url_without_authority() {
        assert!(ApiEndpoint::from_inputs("https:///api/v3", "").is_err());
    }

    #[wasm_bindgen_test]
    fn resolves_current_repository_target_by_default() {
        let target = InstallationTarget::resolve(&input()).unwrap();

        assert_eq!(
            target.installation_paths(),
            vec!["/repos/owner/current/installation".to_string()]
        );
        assert_eq!(target.repository_names(), Some(vec!["current".to_string()]));
    }

    #[wasm_bindgen_test]
    fn resolves_owner_target_without_repository_scope() {
        let mut input = input();
        input.owner = "octo-org".to_string();

        let target = InstallationTarget::resolve(&input).unwrap();

        assert_eq!(
            target.installation_paths(),
            vec![
                "/orgs/octo-org/installation".to_string(),
                "/users/octo-org/installation".to_string()
            ]
        );
        assert_eq!(target.repository_names(), None);
    }

    #[wasm_bindgen_test]
    fn resolves_repository_list_against_owner() {
        let mut input = input();
        input.owner = "octo-org".to_string();
        input.repositories = "repo1, octo-org/repo2\nrepo3".to_string();

        let target = InstallationTarget::resolve(&input).unwrap();

        assert_eq!(
            target.installation_paths(),
            vec!["/repos/octo-org/repo1/installation".to_string()]
        );
        assert_eq!(
            target.repository_names(),
            Some(vec![
                "repo1".to_string(),
                "repo2".to_string(),
                "repo3".to_string()
            ])
        );
    }

    #[wasm_bindgen_test]
    fn rejects_enterprise_with_repository_scope() {
        let mut input = input();
        input.enterprise = "octo-enterprise".to_string();
        input.repositories = "repo1".to_string();

        assert!(InstallationTarget::resolve(&input).is_err());
    }

    #[wasm_bindgen_test]
    fn rejects_repository_owner_mismatch() {
        let error = parse_repository("octo-org", "other-org/repo").unwrap_err();

        assert_eq!(
            error.to_string(),
            "repository 'other-org/repo' includes owner 'other-org', which does not match 'octo-org'"
        );
    }
}
