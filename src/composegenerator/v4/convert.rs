use serde_json::{json, Map, Value};

use crate::composegenerator::compose::types::{ComposeSpecification, Service};
use crate::composegenerator::permissions;
use crate::composegenerator::utils::{get_main_container, validate_cmd, validate_port_map_app};
use crate::composegenerator::v4::types;
use crate::utils::{find_env_vars, flatten};
use std::collections::HashMap;

use super::types::{FinalResult, PortMapElement};

fn configure_ports(
    containers: &HashMap<String, types::Container>,
    main_container: &str,
    output: &mut ComposeSpecification,
    port_map: &HashMap<String, Vec<PortMapElement>>,
) -> Result<(), String> {
    let services = output.services.as_mut().unwrap();
    for service_name in services.clone().keys() {
        let service: &mut Service = services.get_mut(service_name).unwrap();
        let original_definition = containers.get(service_name).unwrap();
        if service_name != main_container && original_definition.port.is_some() {
            return Err(
                "port: is not supported for containers other than the main container".to_string(),
            );
        }
        if port_map.get(service_name).is_none() && original_definition.port.is_some() {
            return Err(format!(
                "Container {} not found or invalid in port map",
                service_name
            )
            .to_string());
        } else if original_definition.port.is_some() {
            let ports = port_map.get(service_name).unwrap();
            for element in ports {
                if original_definition.port.is_some()
                    && element.internal_port == original_definition.port.unwrap()
                {
                    service.ports.push(format!(
                        "{}:{}",
                        element.outside_port, element.internal_port
                    ));
                    break;
                }
            }
        }

        if let Some(required_ports) = &original_definition.required_ports {
            if let Some(tcp_ports) = &required_ports.tcp {
                for port in tcp_ports {
                    service.ports.push(format!("{}:{}", port.0, port.1));
                }
            }
            if let Some(udp_ports) = &required_ports.udp {
                for port in udp_ports {
                    service.ports.push(format!("{}:{}", port.0, port.1));
                }
            }
        }
    }

    Ok(())
}

fn define_ip_addresses(
    app_name: &str,
    containers: &HashMap<String, types::Container>,
    main_container: &str,
    output: &mut ComposeSpecification,
) -> Result<(), String> {
    let services = output.services.as_mut().unwrap();
    for service_name in services.clone().keys() {
        let service: &mut Service = services.get_mut(service_name).unwrap();

        if containers.get(service_name).unwrap().enable_networking {
            service.networks = Some(json!({
                "default": {
                    "ipv4_address": format!("$APP_{}_{}_IP", app_name.to_string().to_uppercase().replace('-', "_"), service_name.to_uppercase().replace('-', "_"))
                }
            }))
        } else if service_name == main_container {
            return Err("Network can not be disabled for the main container".to_string());
        }
    }

    Ok(())
}

fn validate_service(
    app_name: &str,
    permissions: &[String],
    service: &types::Container,
    result: &mut Service,
) -> Result<(), String> {
    if service.entrypoint.is_some() {
        let entrypoint = service.entrypoint.as_ref().unwrap();
        let validation_result = validate_cmd(app_name, entrypoint, permissions);
        if validation_result.is_err() {
            return Err(validation_result.err().unwrap());
        }
        result.entrypoint = Some(entrypoint.clone());
    }
    if service.command.is_some() {
        let command = service.command.as_ref().unwrap();
        let validation_result = validate_cmd(app_name, command, permissions);
        if validation_result.is_err() {
            return Err(validation_result.err().unwrap());
        }
        result.command = Some(command.clone());
    }
    if service.environment.is_some() {
        result.environment = Some(HashMap::<String, String>::new());
        let result_env = result.environment.as_mut().unwrap();
        let env = service.environment.as_ref().unwrap();
        for value in env {
            let val = value.1;
            let env_vars = find_env_vars(val);
            for env_var in env_vars {
                if !permissions::is_allowed_by_permissions(app_name, &env_var, permissions) {
                    return Err(format!("Env var {} not allowed by permissions", env_var));
                }
            }
            result_env.insert(value.0.clone(), val.clone());
        }
    }
    Ok(())
}

fn convert_volumes(
    containers: &HashMap<String, types::Container>,
    permissions: &[String],
    output: &mut ComposeSpecification,
) -> Result<(), String> {
    let services = output.services.as_mut().unwrap();
    for service_name in services.clone().keys() {
        let service: &mut Service = services.get_mut(service_name).unwrap();
        let original_definition = containers.get(service_name).unwrap();
        if let Some(mounts) = &original_definition.mounts {
            if let Some(data_mounts) = &mounts.data {
                for mount in data_mounts {
                    if mount.0.contains("..") {
                        return Err(
                            "A data dir to mount is not allowed to contain '..'".to_string()
                        );
                    }
                    let mount_host_dir: String = if !mount.0.starts_with('/') {
                        "/".to_owned() + mount.0
                    } else {
                        mount.0.clone()
                    };
                    service
                        .volumes
                        .push(format!("${{APP_DATA}}{}:{}", mount_host_dir, mount.1));
                }
            }

            if let Some(bitcoin_mount) = &mounts.bitcoin {
                if !permissions.contains(&"bitcoind".to_string()) {
                    return Err(
                        "bitcoin mount defined by container without Bitcoin permissions"
                            .to_string(),
                    );
                }
                service
                    .volumes
                    .push(format!("${{BITCOIN_DATA_DIR}}:{}", bitcoin_mount));
            }

            if let Some(lnd_mount) = &mounts.lnd {
                if !permissions.contains(&"lnd".to_string()) {
                    return Err(
                        "lnd mount defined by container without LND permissions".to_string()
                    );
                }
                service
                    .volumes
                    .push(format!("${{LND_DATA_DIR}}:{}", lnd_mount));
            }

            if let Some(c_lightning_mount) = &mounts.c_lightning {
                if !permissions.contains(&"c-lightning".to_string()) {
                    return Err(
                        "c-lightning mount defined by container without Core Lightning permissions"
                            .to_string(),
                    );
                }
                service
                    .volumes
                    .push(format!("${{C_LIGHTNING_DATA_DIR}}:{}", c_lightning_mount));
            }
        }
    }

    Ok(())
}

fn get_hidden_services(
    app_name: &str,
    containers: &HashMap<String, types::Container>,
    main_container: &str,
) -> String {
    let mut result = String::new();
    for service_name in containers.clone().keys() {
        let original_definition = containers.get(service_name).unwrap();
        if original_definition.network_mode == Some("host".to_string()) {
            continue;
        }
        let app_name_slug = app_name.to_lowercase().replace('_', "-");
        let service_name_slug = service_name.to_lowercase().replace('_', "-");
        if service_name == main_container {
            let hidden_service_string = format!("HiddenServiceDir /var/lib/tor/app-{}\nHiddenServicePort 80 <app-{}-{}-ip>:<{}-main-port>\n", app_name_slug, app_name_slug, service_name_slug, service_name_slug);
            result += &hidden_service_string.as_str();
        }
        if let Some(hidden_services) = &original_definition.hidden_services {
            match hidden_services {
                types::HiddenServices::PortMap(simple_map) => {
                    if service_name != main_container {
                        let hidden_service_string = format!(
                            "HiddenServiceDir /var/lib/tor/app-{}-{}\n",
                            app_name_slug, service_name_slug
                        );
                        result += &hidden_service_string.as_str();
                    }
                    for port in simple_map {
                        let port_string = format!(
                            "HiddenServicePort {} <app-{}-{}-ip>:{}\n",
                            port.0, app_name_slug, service_name_slug, port.1
                        );
                        result += &port_string.as_str();
                    }
                }
                types::HiddenServices::LayeredMap(layered_map) => {
                    for element in layered_map {
                        let hidden_service_string = format!(
                            "HiddenServiceDir /var/lib/tor/app-{}-{}\n",
                            app_name_slug,
                            element.0.to_lowercase().replace('_', "-")
                        );
                        result += &hidden_service_string.as_str();
                        for port in element.1 {
                            let port_string = format!(
                                "HiddenServicePort {} <app-{}-{}-ip>:{}\n",
                                port.0, app_name_slug, service_name_slug, port.1
                            );
                            result += &port_string.as_str();
                        }
                    }
                }
            }
        }
    }

    result
}

pub fn convert_config(
    app_name: &str,
    app: types::AppYml,
    services: Vec<&str>,
    port_map: &Map<String, Value>,
) -> Result<FinalResult, String> {
    let mut spec: ComposeSpecification = ComposeSpecification {
        // Version is deprecated in the latest compose and should no longer be used
        version: None,
        services: Some(HashMap::new()),
        configs: None,
        name: None,
        networks: None,
        secrets: None,
        volumes: None,
    };
    let spec_services = spec.services.get_or_insert(HashMap::new());
    let permissions = flatten(app.metadata.permissions.clone());
    // Copy all properties that are the same in docker-compose.yml and need no or only a simple validation
    'service_loop: for service_name in app.services.keys() {
        for dependency in app.services[service_name]
            .requires
            .as_ref()
            .unwrap_or(&Vec::<String>::new())
        {
            if !services.contains(&dependency.as_str()) {
                log::debug!(
                    "Service {} depends on {}, which is not installed",
                    service_name,
                    dependency
                );
                continue 'service_loop;
            }
        }
        let service = app.services[service_name].clone();
        let base_result = Service {
            image: Some(service.image.clone()),
            restart: service.restart.clone(),
            stop_grace_period: service.stop_grace_period.clone(),
            stop_signal: service.stop_signal.clone(),
            user: service.user.clone(),
            init: service.init,
            network_mode: service.network_mode.clone(),
            depends_on: service.depends_on.clone().unwrap_or_default(),
            ports: Vec::new(),
            volumes: Vec::new(),
            ..Default::default()
        };
        spec_services.insert(service_name.to_string(), base_result);
        let validation_result = validate_service(
            app_name,
            &permissions,
            &service,
            spec_services.get_mut(service_name).unwrap(),
        );
        if validation_result.is_err() {
            return Err(validation_result.err().unwrap());
        }
    }

    // We can now finalize the process by parsing some of the remaining values
    let main_service_name = get_main_container(&spec);

    if main_service_name.is_err() {
        return Err(main_service_name.err().unwrap());
    }

    let converted_port_map = validate_port_map_app(port_map);
    if converted_port_map.is_err() {
        return Err(converted_port_map.err().unwrap());
    }

    let convert_result = configure_ports(
        &app.services,
        main_service_name.as_ref().unwrap().as_str(),
        &mut spec,
        &converted_port_map.unwrap(),
    );

    if convert_result.is_err() {
        return Err(convert_result.err().unwrap());
    }

    let ip_address_result = define_ip_addresses(
        app_name,
        &app.services,
        main_service_name.as_ref().unwrap().as_str(),
        &mut spec,
    );

    if ip_address_result.is_err() {
        return Err(ip_address_result.err().unwrap());
    }

    let volumes_result = convert_volumes(&app.services, &permissions, &mut spec);

    if volumes_result.is_err() {
        return Err(volumes_result.err().unwrap());
    }

    let result = FinalResult {
        spec,
        new_tor_entries: get_hidden_services(
            app_name,
            &app.services,
            main_service_name.as_ref().unwrap().as_str(),
        ),
    };

    // And we're done
    Ok(result)
}
