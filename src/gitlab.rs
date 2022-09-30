use gitlab::AsyncGitlab;
use semver::Version;
use serde::Deserialize;
use url::Url;
use gitlab::api::AsyncQuery;
use gitlab::api::projects::repository::tags::Tags;

// The API has more data, but we only need this
#[derive(Debug, Deserialize)]
struct Tag {
    name: String,
}

pub async fn check_updates(
    gitlab: &AsyncGitlab,
    repo: String,
    current_version: &Version,
    include_pre: bool,
) -> Result<String, String> {
    let endpoint = Tags::builder().project(repo).build();
    if let Err(err) = endpoint {
        return Err(err.to_string());
    }
    let endpoint = endpoint.unwrap();
    let result = endpoint.query_async(gitlab).await;
    if let Err(tag_error) = result {
        return Err(tag_error.to_string());
    }
    let tags: Vec<Tag> = result.unwrap();
    for tag in tags {
        let tag = tag.name;
        // Remove the v prefix if it exists
        let tag = tag.trim_start_matches('v');
        let version = Version::parse(tag);
        if let Err(err) = version {
            eprintln!("Error while parsing tag {}: {}", tag, err);
            continue;
        }
        let version = version.unwrap();
        if (include_pre || version.pre.is_empty()) && &version > current_version {
            return Ok(tag.to_string());
        }
    }

    Err("No update found".to_string())
}

// Given a GitLab repository path, return the name of the GitLab instance
// And the repo path
pub fn get_repo_path(url: &str) -> Option<(String, String)> {
    let url = Url::parse(url);
    if url.is_err() || !url.as_ref().unwrap().has_host() {
        return None;
    }
    let url = url.unwrap();
    let segments: Vec<&str> = url.path_segments().unwrap_or_else(|| "".split(' ')).collect();
    if segments.len() < 2{
        return None;
    }
    Some((url.host().unwrap().to_string(), segments.join("/")))
}

#[cfg(test)]
mod test {
    use super::get_repo_path;

    #[test]
    fn test_get_repo_path() {
        let repo_path = "https://gitlab.com/AaronDewes/repo";
        let repo_path = get_repo_path(repo_path);
        assert_eq!(
            repo_path,
            Some(("gitlab.com".to_string(), "AaronDewes/repo".to_string()))
        );
    }

    #[test]
    fn test_get_repo_path_invalid() {
        let repo_path = "https://gitlab.com/justuser";
        let repo_path = get_repo_path(repo_path);
        assert!(repo_path.is_none());
    }
}
