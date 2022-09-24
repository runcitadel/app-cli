use crate::composegenerator::{AppYmlFile, v4::update::update_container};
use crate::github::{get_repo_path, check_updates};

pub async fn update_app(app: &mut AppYmlFile, include_pre: bool) {
    let docker = bollard::Docker::connect_with_local_defaults().unwrap();
    match app {
        AppYmlFile::V4(app) => {
            let update_containers = app.metadata.update_containers.clone().unwrap_or(vec!["main".to_string(), "web".to_string()]);
            let repo = get_repo_path(app.metadata.repo.values().next().unwrap());
            if repo.is_none() {
                eprintln!("Could not parse repo path");
                return;
            }
            let current_version = app.metadata.version.clone();
            let current_version = semver::Version::parse(&current_version);
            if current_version.is_err() {
                eprintln!("Could not parse current version");
                return;
            }
            let current_version = current_version.unwrap();
            let (owner, repo) = repo.unwrap();
            let latest_tag = check_updates(&owner, &repo, &current_version, include_pre).await;
            if let Err(error) = latest_tag {
                eprintln!("Failed to get latest release: {}", error);
                return;
            }
            let latest_tag = latest_tag.unwrap();

            let mut failure = false;
            for (name, service) in app.services.iter_mut() {
                if !update_containers.contains(name) {
                    continue;
                }
                let update_result = update_container(service, &latest_tag, &docker).await;
                if let Err(error) = update_result {
                    failure = true;
                    eprintln!("{}", error);
                }
            }
            if failure {
                eprintln!("Failed to update some containers");
            } else {
                app.metadata.version = latest_tag;
            }
        }
    }
}