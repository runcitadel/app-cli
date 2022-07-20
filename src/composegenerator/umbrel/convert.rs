use std::collections::HashMap;

use crate::composegenerator::umbrel::types::Metadata;
use crate::composegenerator::v4::types::{Metadata as V4Metadata, Permissions, AppYml, Container, Mounts};
use crate::map;

pub fn convert_metadata(metadata: Metadata) -> V4Metadata {
    let deps: Vec<Permissions> = metadata.dependencies.into_iter().map(| dep | -> Permissions {
        if dep == "lightning" {
            return Permissions::OneDependency("lnd".to_string());
        }
        if dep == "bitcoin" {
            return Permissions::OneDependency("bitcoind".to_string());
        }
        if dep == "electrs" {
            return Permissions::OneDependency("electrum".to_string());
        }
        return Permissions::OneDependency(dep);
    }).collect();
    V4Metadata {
        name: metadata.name,
        version: metadata.version,
        repo: map! {
            "Public" => metadata.repo
        },
        support: metadata.support,
        category: metadata.category,
        tagline: metadata.tagline,
        permissions: deps,
        developers: map! {
            metadata.developer => metadata.website
        },
        gallery: metadata.gallery,
        path: metadata.path,
        default_password: if metadata.deterministic_password {
            Some("$APP_SEED".to_string())
        } else {
            metadata.default_password
        },
        tor_only: metadata.tor_only,
        update_containers: None,
        description: metadata.description,
    }
}

pub fn convert_compose(compose: crate::composegenerator::compose::types::ComposeSpecification, metadata: Metadata) -> AppYml {
    let services = compose.services.unwrap();
    let mut result_services: HashMap<String, Container> = HashMap::new(); 
    for service in services {
        let service_name = service.0;
        let service_def = service.1;
        // We don't have an app_proxy
        if service_name == "app_proxy" {
            continue
        }
        let mut mounts: Option<Mounts> = Some(Mounts {
            bitcoin: None,
            lnd: None,
            c_lightning: None,
            data: Some(HashMap::new()),
        });
        for volume in service_def.volumes {
            // Convert mounts using env vars to real mounts
            // For example, if a volume is "${APP_DATA_DIR}/thing:/data",
            // we add set "/thing" of the mounts.data hashmap to "/data"
            let split = volume.split(":").collect::<Vec<&str>>();
            if split.len() != 2 {
                continue
            }
            let volume_name = split[0];
            let volume_path = split[1];
            if volume_name.starts_with("${APP_DATA_DIR}") {
                let volume_name_without_prefix = volume_name.replace("${APP_DATA_DIR}", "");
                let volume_name_without_prefix = volume_name_without_prefix.trim_start_matches("/");
                mounts.as_mut().unwrap().data.as_mut().unwrap().insert(volume_name_without_prefix.to_string(), volume_path.to_string());
            } else if volume_name.starts_with("${APP_LIGHTNING_NODE_DATA_DIR}") {
                mounts.as_mut().unwrap().lnd = Some(volume_path.to_string());
            } else if volume_name.starts_with("${APP_BITCOIN_DATA_DIR}") {
                mounts.as_mut().unwrap().bitcoin = Some(volume_path.to_string());
            } else if volume_name.starts_with("${APP_CORE_LIGHTNING_REST_CERT_DIR}") {
                mounts.as_mut().unwrap().c_lightning = Some("Please set this yourself, I could not automatically check this.".to_string());
            }
        }
        // Loop through environment (a Option<hashmap>) and in all values, make these replacements
        // APP_BITCOIN_NETWORK -> BITCOIN_NETWORK
        // APP_PASSWORD -> APP_SEED
        // APP_LIGHTNING_NODE_GRPC_PORT -> LND_GRPC_PORT
        // APP_LIGHTNING_NODE_REST_PORT -> LND_REST_PORT
        // APP_LIGHTNING_NODE_IP -> LND_IP
        let mut env: Option<HashMap<String, String>> = Some(HashMap::new());
        for (key, value) in service_def.environment.unwrap() {
            let mut new_value = value.clone();
            if value.contains("APP_BITCOIN_NETWORK") {
                new_value = value.replace("APP_BITCOIN_NETWORK", "BITCOIN_NETWORK");
            }
            if value.contains("APP_PASSWORD") {
                new_value = value.replace("APP_PASSWORD", "APP_SEED");
            }
            if value.contains("APP_LIGHTNING_NODE_GRPC_PORT") {
                new_value = value.replace("APP_LIGHTNING_NODE_GRPC_PORT", "LND_GRPC_PORT");
            }
            if value.contains("APP_LIGHTNING_NODE_REST_PORT") {
                new_value = value.replace("APP_LIGHTNING_NODE_REST_PORT", "LND_REST_PORT");
            }
            if value.contains("APP_LIGHTNING_NODE_IP") {
                new_value = value.replace("APP_LIGHTNING_NODE_IP", "LND_IP");
            }
            env.as_mut().unwrap().insert(key, new_value);
        }
        let new_service = Container {
            image: service_def.image.unwrap(),
            user: service_def.user,
            stop_grace_period: service_def.stop_grace_period,
            stop_signal: service_def.stop_signal,
            depends_on: service_def.depends_on,
            network_mode: service_def.network_mode,
            restart: service_def.restart,
            init: service_def.init,
            extra_hosts: service_def.extra_hosts,
            entrypoint: service_def.entrypoint,
            command: service_def.command,
            environment: env,
            port: if service_name == "main" || service_name == "web" { Some(metadata.port) } else { None },
            port_priority: None,
            required_ports: None,
            mounts: mounts,
            enable_networking: if service_def.networks.is_some() { None } else { Some(false) },
            hidden_services: None,
        };
        result_services.insert(service_name, new_service);
    }
    AppYml {
        citadel_version: 4,
        metadata: convert_metadata(metadata),
        services: result_services,
    }
}
