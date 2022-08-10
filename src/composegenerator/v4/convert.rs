use serde_json::{json, Map, Value};

use crate::composegenerator::compose::types::{ComposeSpecification, Service};
use crate::composegenerator::permissions;
use crate::composegenerator::utils::{
    get_host_port, get_main_container, validate_cmd, validate_port_map_app,
};
use crate::composegenerator::v4::types;
use crate::utils::{find_env_vars, flatten};
use std::collections::HashMap;

use super::types::{FinalResult, PortMapElement};

fn configure_ports(
    containers: &HashMap<String, types::Container>,
    main_container: &str,
    output: &mut ComposeSpecification,
    port_map: &Option<HashMap<String, Vec<PortMapElement>>>,
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

        if let Some(internal_port) = original_definition.port {
            if service_name != main_container {
                return Err(
                    "port: is not supported for containers other than the main container"
                        .to_string(),
                );
            }
            let outside_port: Option<&PortMapElement>;
            let fake_port = PortMapElement {
                internal_port,
                outside_port: internal_port,
                dynamic: false,
            };
            if let Some(real_port_map) = port_map {
                if real_port_map.get(service_name).is_none() {
                    return Err(format!(
                        "Container {} not found or invalid in port map",
                        service_name
                    ));
                }
                let ports = real_port_map.get(service_name).unwrap();
                outside_port = get_host_port(ports, internal_port);
            } else {
                outside_port = Some(&fake_port);
            }
            if let Some(port_map_elem) = outside_port {
                service
                    .ports
                    .push(format!("{}:{}", port_map_elem.outside_port, internal_port));
                break;
            } else {
                return Err("Main container port not found in port map".to_string());
            }
        } else if service_name == main_container {
            return Err("A port is required for the main container".to_string());
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

        if containers
            .get(service_name)
            .unwrap()
            .enable_networking
            .unwrap_or(true)
        {
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
                if !permissions::is_allowed_by_permissions(app_name, env_var, permissions) {
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
                        .push(format!("${{APP_DATA_DIR}}{}:{}", mount_host_dir, mount.1));
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
            result += hidden_service_string.as_str();
        }
        if let Some(hidden_services) = &original_definition.hidden_services {
            match hidden_services {
                types::HiddenServices::PortMap(simple_map) => {
                    if service_name != main_container {
                        let hidden_service_string = format!(
                            "HiddenServiceDir /var/lib/tor/app-{}-{}\n",
                            app_name_slug, service_name_slug
                        );
                        result += hidden_service_string.as_str();
                    }
                    for port in simple_map {
                        let port_string = format!(
                            "HiddenServicePort {} <app-{}-{}-ip>:{}\n",
                            port.0, app_name_slug, service_name_slug, port.1
                        );
                        result += port_string.as_str();
                    }
                }
                types::HiddenServices::LayeredMap(layered_map) => {
                    for element in layered_map {
                        let hidden_service_string = format!(
                            "HiddenServiceDir /var/lib/tor/app-{}-{}\n",
                            app_name_slug,
                            element.0.to_lowercase().replace('_', "-")
                        );
                        result += hidden_service_string.as_str();
                        for port in element.1 {
                            let port_string = format!(
                                "HiddenServicePort {} <app-{}-{}-ip>:{}\n",
                                port.0, app_name_slug, service_name_slug, port.1
                            );
                            result += port_string.as_str();
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
    port_map: &Option<&Map<String, Value>>,
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
    for service_name in app.services.keys() {
        let service = app.services[service_name].clone();
        let mut base_result = Service {
            image: Some(service.image.clone()),
            restart: service.restart.clone(),
            stop_grace_period: service.stop_grace_period.clone(),
            stop_signal: service.stop_signal.clone(),
            user: service.user.clone(),
            init: service.init,
            network_mode: service.network_mode.clone(),
            depends_on: service.depends_on.clone(),
            ports: Vec::new(),
            volumes: Vec::new(),
            ..Default::default()
        };
        if let Some(extra_hosts) = service.extra_hosts.as_ref() {
            base_result.extra_hosts = Some(extra_hosts.clone());
        }
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
    let main_service = get_main_container(&spec);

    if main_service.is_err() {
        return Err(main_service.err().unwrap());
    }

    let main_service_name = main_service.as_ref().unwrap().as_str();
    let mut converted_port_map: Option<HashMap<String, Vec<PortMapElement>>> = None;
    if let Some(real_port_map) = port_map {
        let conversion_result = validate_port_map_app(real_port_map);
        match conversion_result {
            Err(conversion_error) => {
                return Err(conversion_error);
            }
            Ok(conversion_result) => {
                converted_port_map = Some(conversion_result);
            }
        }
    }

    let convert_result = configure_ports(
        &app.services,
        main_service_name,
        &mut spec,
        &converted_port_map,
    );

    if convert_result.is_err() {
        return Err(convert_result.err().unwrap());
    }

    let ip_address_result =
        define_ip_addresses(app_name, &app.services, main_service_name, &mut spec);

    if ip_address_result.is_err() {
        return Err(ip_address_result.err().unwrap());
    }

    let volumes_result = convert_volumes(&app.services, &permissions, &mut spec);

    if volumes_result.is_err() {
        return Err(volumes_result.err().unwrap());
    }

    if app.services.get(main_service_name).unwrap().port.is_none() {
        return Err("Main container does not declare port".to_string());
    }

    let main_port = app.services.get(main_service_name).unwrap().port.unwrap();

    let mut main_port_host: Option<u16> = None;
    if let Some(converted_map) = converted_port_map {
        main_port_host = Some(
            get_host_port(converted_map.get(main_service_name).unwrap(), main_port)
                .unwrap()
                .outside_port,
        );
    }

    let mut metadata = app.metadata.clone();
    metadata.id = Some(app_name.to_string());
    let result = FinalResult {
        spec,
        new_tor_entries: get_hidden_services(app_name, &app.services, main_service_name),
        port: main_port_host.unwrap_or(main_port),
        metadata,
    };

    // And we're done
    Ok(result)
}

#[cfg(test)]
mod test {
    use crate::{
        composegenerator::{
            compose::types::{ComposeSpecification, Service},
            v4::{
                convert::convert_config,
                types::AppYml,
                types::Metadata,
                types::Permissions,
                types::{Container, FinalResult},
            },
        },
        map,
    };

    use pretty_assertions::assert_eq;
    use serde_json::json;

    #[test]
    fn test_simple_app() {
        let example_app = AppYml {
            citadel_version: 4,
            metadata: Metadata {
                // To test if this is being overwritten
                id: Some("a-fake-id-that-should-be-ignored".to_string()),
                name: "Example app".to_string(),
                version: "1.0.0".to_string(),
                category: "Example category".to_string(),
                tagline: "The only example app for Citadel you will ever need".to_string(),
                developers: map! {
                    "Citadel team".to_string() => "runcitadel.space".to_string()
                },
                permissions: vec![Permissions::OneDependency("lnd".to_string())],
                repo: map! {
                    "Example repo".to_string() => "https://github.com/runcitadel/app-cli".to_string()
                },
                support: "https://t.me/citadeldevelopers".to_string(),
                description: "This is an example app that provides multiple features that you need on your node. These features include:\n\n- Example\n- Example\n- Example".to_string(),
                ..Default::default()
            },
            services: map! {
                "main" => Container {
                    image: "ghcr.io/runcitadel/example:main".to_string(),
                    user: Some("1000:1000".to_string()),
                    depends_on: Some(vec!["database".to_string()]),
                    port: Some(3000),
                    ..Default::default()
                },
                "database" => Container {
                    image: "ghcr.io/runcitadel/example-db:main".to_string(),
                    user: Some("1000:1000".to_string()),
                    ..Default::default()
                }
            }
        };
        let result = convert_config("example-app", example_app, &None);
        assert!(result.is_ok());
        let expected_result = FinalResult {
            port: 3000,
            new_tor_entries: "HiddenServiceDir /var/lib/tor/app-example-app\nHiddenServicePort 80 <app-example-app-main-ip>:<main-main-port>\n".to_string(),
            spec: ComposeSpecification {
                services: Some(map! {
                    "main" => Service {
                        image: Some("ghcr.io/runcitadel/example:main".to_string()),
                        user: Some("1000:1000".to_string()),
                        depends_on: Some(vec!["database".to_string()]),
                        ports: vec!["3000:3000".to_string()],
                        networks: Some(json!({
                            "default": Some(json!({
                                "ipv4_address": "$APP_EXAMPLE_APP_MAIN_IP".to_string()
                            }))
                        })),
                        ..Default::default()
                    },
                    "database" => Service {
                        image: Some("ghcr.io/runcitadel/example-db:main".to_string()),
                        user: Some("1000:1000".to_string()),
                        networks: Some(json!({
                            "default": Some(json!({
                                "ipv4_address": "$APP_EXAMPLE_APP_DATABASE_IP".to_string()
                            }))
                        })),
                        ..Default::default()
                    }
                }),
                ..Default::default()

            },
            metadata: Metadata {
                id: Some("example-app".to_string()),
                name: "Example app".to_string(),
                version: "1.0.0".to_string(),
                category: "Example category".to_string(),
                tagline: "The only example app for Citadel you will ever need".to_string(),
                developers: map! {
                    "Citadel team".to_string() => "runcitadel.space".to_string()
                },
                permissions: vec![Permissions::OneDependency("lnd".to_string())],
                repo: map! {
                    "Example repo".to_string() => "https://github.com/runcitadel/app-cli".to_string()
                },
                support: "https://t.me/citadeldevelopers".to_string(),
                description: "This is an example app that provides multiple features that you need on your node. These features include:\n\n- Example\n- Example\n- Example".to_string(),
                ..Default::default()
            },
        };
        assert_eq!(expected_result, result.unwrap());
    }
}
