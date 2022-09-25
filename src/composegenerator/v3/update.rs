use bollard::{image::CreateImageOptions, Docker};

use super::types::SchemaItemContainers;
use futures_util::stream::TryStreamExt;

pub async fn get_hash(
    container: &str,
    docker: &Docker,
) -> Result<String, bollard::errors::Error> {
    println!("Pulling {}...", container);
    docker
        .create_image(
            Some(CreateImageOptions {
                from_image: container,
                ..Default::default()
            }),
            None,
            None,
        )
        .try_collect::<Vec<_>>().await?;
    let hash = docker.inspect_image(container).await?;
    let digests = hash.repo_digests.expect("No digest found!");
    let result = digests.first().expect("No digest found!");

    Ok(result.to_owned().split('@').last().unwrap().to_owned())
}

pub async fn update_container(container: &mut SchemaItemContainers, to_version: &String, docker: &Docker) -> Result<(), bollard::errors::Error> {
    let image = &container.image;
    let image_without_tag = image.split(':').next().expect("Splitting failed");
    let mut new_tag = image_without_tag.to_owned() + ":" + to_version;
    let new_hash = get_hash(&new_tag, docker).await;
    let hash: String;
    if let Ok(new_image) = new_hash {
        hash = new_image;
    } else {
        new_tag = image_without_tag.to_owned() + ":v" + to_version;
        let new_image = get_hash(&new_tag, docker).await?;
        hash = new_image;
    }
    container.image = new_tag + "@" + &hash;
    Ok(())
}
