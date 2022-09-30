use gitlab::Gitlab;

use super::composegenerator::types::Metadata;
use super::github;

pub async fn check_updates(
    metadata: &Metadata,
    include_pre: bool,
    token: Option<String>,
) -> Result<String, String> {
    let current_version = metadata.version.clone();
    let current_version = semver::Version::parse(&current_version);
    if current_version.is_err() {
        return Err("Could not parse current version".to_string());
    }
    let current_version = current_version.unwrap();
    match metadata.version_control.clone().unwrap_or_else(|| "github".to_string()).to_lowercase().as_str() {
        "github" => {
            if let Some(gh_token) = token {
                octocrab::initialise(octocrab::OctocrabBuilder::new().personal_token(gh_token))
                    .expect("Failed to initialise octocrab");
            }
            let repo_path = github::get_repo_path(
                metadata
                    .repo
                    .values()
                    .next()
                    .expect("Missing repo for app")
                    .as_str(),
            );
            if repo_path.is_none() {
                return Err("No repo path found".to_string());
            }
            let (owner, repo) = repo_path.unwrap();
            super::github::check_updates(&owner, &repo, &current_version, include_pre).await
        }
        "gitlab" => {
            let repo_path = super::gitlab::get_repo_path(
                metadata
                    .repo
                    .values()
                    .next()
                    .expect("Missing repo for app")
                    .as_str(),
            );
            if repo_path.is_none() {
                return Err("No repo path found".to_string());
            }
            let (gitlab_server, repo) = repo_path.unwrap();
            let client = Gitlab::builder(gitlab_server, token.unwrap_or_else(|| "".to_string()))
                .build_async()
                .await;
            if let Err(client_err) = client {
                return Err(client_err.to_string());
            }
            let client = client.unwrap();
            super::gitlab::check_updates(&client, repo, &current_version, include_pre).await
        },
        _ => Err("Version control system not supported".to_string())
    }
}
