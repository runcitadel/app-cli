use std::collections::HashMap;

use crate::composegenerator::compose::types::{Command, EnvVars, StringOrIntOrBool};
use crate::composegenerator::types::{Metadata as CitadelMetadata, Permissions};
use crate::composegenerator::umbrel::types::Metadata;
use crate::composegenerator::v4::types::{AppYml, Container, Mounts};
use crate::map;

pub fn convert_metadata(metadata: Metadata) -> CitadelMetadata {
    let deps: Vec<Permissions> = metadata
        .dependencies
        .into_iter()
        .map(|dep| -> Permissions {
            match dep.as_str() {
                "lightning" => Permissions::OneDependency("lnd".to_string()),
                "bitcoin" => Permissions::OneDependency("bitcoind".to_string()),
                "electrs" => Permissions::OneDependency("electrum".to_string()),
                _ => Permissions::OneDependency(dep),
            }
        })
        .collect();
    CitadelMetadata {
        id: None,
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
        implements: None,
    }
}

fn replace_env_vars(mut string: String) -> String {
    if string.contains("APP_BITCOIN_NETWORK") {
        string = string.replace("APP_BITCOIN_NETWORK", "BITCOIN_NETWORK");
    }
    if string.contains("APP_LIGHTNING_NODE_GRPC_PORT") {
        string = string.replace("APP_LIGHTNING_NODE_GRPC_PORT", "LND_GRPC_PORT");
    }
    if string.contains("APP_LIGHTNING_NODE_REST_PORT") {
        string = string.replace("APP_LIGHTNING_NODE_REST_PORT", "LND_REST_PORT");
    }
    if string.contains("APP_LIGHTNING_NODE_IP") {
        string = string.replace("APP_LIGHTNING_NODE_IP", "LND_IP");
    }
    if string.contains("APP_ELECTRS_NODE_IP") {
        string = string.replace("APP_ELECTRS_NODE_IP", "ELECTRUM_IP");
    }
    if string.contains("APP_ELECTRS_NODE_PORT") {
        string = string.replace("APP_ELECTRS_NODE_PORT", "ELECTRUM_PORT");
    }
    string
}

pub fn convert_compose(
    compose: crate::composegenerator::compose::types::ComposeSpecification,
    metadata: Metadata,
) -> AppYml {
    let services = compose.services.unwrap();
    let mut result_services: HashMap<String, Container> = HashMap::new();
    let has_main = services.contains_key("main");
    let mut deps = Vec::<String>::new();
    for service in services {
        let mut service_name = service.0;
        let service_def = service.1;
        // We don't have an app_proxy
        if service_name == "app_proxy" {
            continue;
        }
        if service_name == "web" && !has_main {
            service_name = "main".to_string();
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
            let split = volume.split(':').collect::<Vec<&str>>();
            if split.len() != 2 && split.len() != 3 {
                continue;
            }
            let volume_name = split[0];
            let volume_path = split[1];
            if volume_name.contains("${APP_DATA_DIR}") || volume_name.contains("$APP_DATA_DIR") {
                let volume_name_without_prefix = volume_name
                    .replace("${APP_DATA_DIR}", "")
                    .replace("$APP_DATA_DIR", "");
                let volume_name_without_prefix = volume_name_without_prefix.trim_start_matches('/');
                mounts.as_mut().unwrap().data.as_mut().unwrap().insert(
                    volume_name_without_prefix.to_string(),
                    volume_path.to_string(),
                );
            } else if volume_name.contains("APP_LIGHTNING_NODE_DATA_DIR") {
                mounts.as_mut().unwrap().lnd = Some(volume_path.to_string());
            } else if volume_name.contains("APP_BITCOIN_DATA_DIR") {
                mounts.as_mut().unwrap().bitcoin = Some(volume_path.to_string());
            } else if volume_name.contains("APP_CORE_LIGHTNING_REST_CERT_DIR") {
                mounts.as_mut().unwrap().c_lightning = Some(
                    "Please set this yourself, I could not automatically check this.".to_string(),
                );
            }
        }
        // Loop through environment (a Option<hashmap>) and in all values, make these replacements
        // APP_BITCOIN_NETWORK -> BITCOIN_NETWORK
        // APP_PASSWORD -> APP_SEED
        // APP_LIGHTNING_NODE_GRPC_PORT -> LND_GRPC_PORT
        // APP_LIGHTNING_NODE_REST_PORT -> LND_REST_PORT
        // APP_LIGHTNING_NODE_IP -> LND_IP
        let mut env: Option<HashMap<String, StringOrIntOrBool>> = Some(HashMap::new());
        let original_env = match service_def.environment {
            Some(env) => match env {
                EnvVars::List(list) => {
                    let mut map = HashMap::<String, StringOrIntOrBool>::new();
                    for val in list {
                        let mut split = val.split('=');
                        map.insert(
                            split.next().expect("Env var invalid").to_string(),
                            StringOrIntOrBool::String(split.next().expect("Env var invalid").to_string()),
                        );
                    }
                    map
                }
                EnvVars::Map(map) => map,
            },
            None => HashMap::<String, StringOrIntOrBool>::new(),
        };
        for (key, value) in original_env {
            let new_value = match value {
                StringOrIntOrBool::String(str) => {
                    let mut new_value = replace_env_vars(str);
                    // If the APP_PASSWORD is also used, there could be a conflict otherwise
                    // For apps which don't use APP_PASSWORD, this can be reverted
                    if new_value.contains("APP_SEED") {
                        new_value = new_value.replace("APP_SEED", "APP_SEED_2");
                    }
                    if new_value.contains("APP_PASSWORD") {
                        new_value = new_value.replace("APP_PASSWORD", "APP_SEED");
                    }
                    StringOrIntOrBool::String(new_value)
                },
                _ => value,
            };
            env.as_mut()
                .unwrap()
                .insert(key, new_value);
        }
        let mut new_cmd: Option<Command> = None;
        if let Some(command) = service_def.command {
            match command {
                Command::SimpleCommand(mut command) => {
                    command = replace_env_vars(command);
                    if command.contains("APP_PASSWORD") {
                        // If the APP_SEED is also used, use APP_SEED_2 instead so the seed and the password are different
                        if command.contains("APP_SEED") {
                            command = command.replace("APP_SEED", "APP_SEED_2");
                        }
                        command = command.replace("APP_PASSWORD", "APP_SEED");
                    }
                    new_cmd = Some(Command::SimpleCommand(command));
                }
                Command::ArrayCommand(values) => {
                    let mut result = Vec::<String>::new();
                    for mut argument in values {
                        argument = replace_env_vars(argument);
                        // If the APP_PASSWORD is also used, there could be a conflict otherwise
                        // For apps which don't use APP_PASSWORD, this can be reverted
                        if argument.contains("APP_SEED") {
                            argument = argument.replace("APP_SEED", "APP_SEED_2");
                        }
                        if argument.contains("APP_PASSWORD") {
                            argument = argument.replace("APP_PASSWORD", "APP_SEED");
                        }
                        result.push(argument);
                    }
                    new_cmd = Some(Command::ArrayCommand(result));
                }
            };
        }
        if let Some(caps) = &service_def.cap_add {
            if caps.contains(&"CAP_NET_ADMIN".to_string()) || caps.contains(&"CAP_NET_RAW".to_string()) {
                deps.push("network".to_string());
            }
        }
        if service_def.network_mode.is_some() {
            deps.push("network".to_string());
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
            command: new_cmd,
            environment: env,
            port: if service_name == "main" || service_name == "web" {
                Some(metadata.port)
            } else {
                None
            },
            port_priority: None,
            required_ports: None,
            mounts,
            enable_networking: if service_def.networks.is_some() {
                None
            } else {
                Some(false)
            },
            hidden_services: None,
            cap_add: service_def.cap_add,
        };
        result_services.insert(service_name, new_service);
    }
    AppYml {
        citadel_version: 4,
        metadata: convert_metadata(metadata),
        services: result_services,
    }
}
