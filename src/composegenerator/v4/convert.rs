use serde_json::{json, Map, Value};

use super::{
    permissions, types,
    types::PortMapElement,
    utils::{get_host_port, get_main_container, validate_cmd, validate_port_map_app},
};
use crate::composegenerator::{compose::types::{
    ComposeSpecification, EnvVars, Service, StringOrIntOrBool,
}, types::Permissions};
use crate::utils::{find_env_vars, flatten};
use std::collections::HashMap;

use crate::composegenerator::types::ResultYml;

fn get_main_port(
    containers: &HashMap<String, types::Container>,
    main_container: &str,
    port_map: &Option<HashMap<String, Vec<PortMapElement>>>,
) -> Result<u16, String> {
    let mut result: u16 = 0;
    for service_name in containers.keys() {
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
            let public_port: Option<&PortMapElement>;
            let fake_port = PortMapElement {
                internal_port,
                public_port: internal_port,
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
                public_port = get_host_port(ports, internal_port);
            } else {
                public_port = Some(&fake_port);
            }
            if public_port.is_some() {
                result = internal_port;
                break;
            } else {
                return Err("Main container port not found in port map".to_string());
            }
        } else if service_name == main_container {
            let empty_vec = Vec::<PortMapElement>::with_capacity(0);
            if let Some(real_port) = port_map.clone()
                .unwrap_or_default()
                .get(service_name)
                .unwrap_or(&empty_vec)
                .iter()
                .find(|elem| elem.dynamic)
            {
                result = real_port.internal_port;
            } else if port_map.is_none() {
                result = 3000;
            } else {
                return Err("A port is required for the main container".to_string());
            }
        }
    }

    Ok(result)
}

fn configure_ports(
    containers: &HashMap<String, types::Container>,
    main_container: &str,
    output: &mut ComposeSpecification,
    port_map: &Option<HashMap<String, Vec<PortMapElement>>>,
) -> Result<(), String> {
    let services = output.services.as_mut().unwrap();
    for (service_name, service) in services {
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
            let public_port: Option<&PortMapElement>;
            let fake_port = PortMapElement {
                internal_port,
                public_port: internal_port,
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
                public_port = get_host_port(ports, internal_port);
            } else {
                public_port = Some(&fake_port);
            }
            if let Some(port_map_elem) = public_port {
                service
                    .ports
                    .push(format!("{}:{}", port_map_elem.public_port, internal_port));
                break;
            } else {
                return Err("Main container port not found in port map".to_string());
            }
        } else if service_name == main_container {
            let empty_vec = Vec::<PortMapElement>::with_capacity(0);
            if port_map.is_some()
                && !port_map
                    .clone()
                    .unwrap()
                    .get(service_name)
                    .unwrap_or(&empty_vec)
                    .iter()
                    .any(|elem| elem.dynamic)
            {
                return Err("A port is required for the main container".to_string());
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
                    service.ports.push(format!("{}:{}/udp", port.0, port.1));
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
    for (service_name, service) in services {
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
    permissions: &mut Vec<String>,
    service: &types::Container,
    replace_env_vars: &HashMap<String, String>,
    result: &mut Service,
) -> Result<(), String> {
    if let Some(entrypoint) = &service.entrypoint {
        let validation_result = validate_cmd(app_name, entrypoint, permissions);
        if validation_result.is_err() {
            return Err(validation_result.err().unwrap());
        }
        result.entrypoint = Some(entrypoint.to_owned());
    }
    if let Some(command) = &service.command {
        let validation_result = validate_cmd(app_name, command, permissions);
        if validation_result.is_err() {
            return Err(validation_result.err().unwrap());
        }
        result.command = Some(command.to_owned());
    }
    if let Some(env) = &service.environment {
        result.environment = Some(EnvVars::Map(HashMap::<String, StringOrIntOrBool>::new()));
        let result_env = result.environment.as_mut().unwrap();
        for value in env {
            let val = match value.1 {
                StringOrIntOrBool::String(val) => {
                    let env_vars = find_env_vars(val);
                    for env_var in &env_vars {
                        if !permissions::is_allowed_by_permissions(app_name, env_var, permissions) {
                            return Err(format!("Env var {} not allowed by permissions", env_var));
                        }
                    }
                    let mut val = val.to_owned();
                    if !env_vars.is_empty() {
                        let to_replace = replace_env_vars
                            .iter()
                            .filter(|(key, _)| env_vars.contains(&key.as_str()));
                        for (env_var, replacement) in to_replace {
                            let syntax_1 = "$".to_string() + env_var;
                            let syntax_2 = format!("${{{}}}", env_var);
                            val = val.replace(&syntax_1, replacement);
                            val = val.replace(&syntax_2, replacement);
                        }
                    }
                    StringOrIntOrBool::String(val)
                }
                StringOrIntOrBool::Int(int) => StringOrIntOrBool::Int(*int),
                StringOrIntOrBool::Bool(bool) => StringOrIntOrBool::Bool(*bool),
            };

            match result_env {
                EnvVars::List(_) => unreachable!(),
                EnvVars::Map(map) => map.insert(value.0.to_owned(), val),
            };
        }
    }
    if service.network_mode.is_some()  {
        if !permissions.contains(&"network".to_string()) {
            // To preserve compatibility, this is only a warning, but we add the permission to the output
            eprintln!("App defines network-mode, but does not request the network permission");
            permissions.push("network".to_string());
        }
        result.network_mode = service.network_mode.to_owned();
    }
    if let Some(caps) = &service.cap_add {
        let mut cap_add = Vec::<String>::new();
        for cap in caps {
            match cap.to_lowercase().as_str() {
                "cap-net-raw" | "cap-net-admin" => {
                    if !permissions.contains(&"network".to_string()) {
                        return Err("App defines a network capability, but does not request the network permission".to_string());
                    }
                    cap_add.push(cap.to_owned());
                },
                _ => {
                    return Err(format!("App defines unknown capability: {}", cap))
                }
            }
        }
        result.cap_add = Some(cap_add);
    }
    Ok(())
}

fn convert_volumes(
    containers: &HashMap<String, types::Container>,
    permissions: &[String],
    output: &mut ComposeSpecification,
) -> Result<(), String> {
    let services = output.services.as_mut().unwrap();
    for (service_name, service) in services {
        let original_definition = containers.get(service_name).unwrap();
        if let Some(mounts) = &original_definition.mounts {
            if let Some(data_mounts) = &mounts.data {
                for (host_path, container_path) in data_mounts {
                    if host_path.contains("..") {
                        return Err(
                            "A data dir to mount is not allowed to contain '..'".to_string()
                        );
                    }
                    let mount_host_dir: String = if !host_path.starts_with('/') {
                        "/".to_owned() + host_path
                    } else {
                        host_path.clone()
                    };
                    service
                        .volumes
                        .push(format!("${{APP_DATA_DIR}}{}:{}", mount_host_dir, container_path));
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
    containers: HashMap<String, types::Container>,
    main_container: &str,
    main_port: u16,
) -> String {
    let mut result = String::new();
    for service_name in containers.keys() {
        let original_definition = containers.get(service_name).unwrap();
        if original_definition.network_mode == Some("host".to_string()) {
            continue;
        }
        let app_name_slug = app_name.to_lowercase().replace('_', "-");
        let service_name_slug = service_name.to_lowercase().replace('_', "-");
        if service_name == main_container {
            let hidden_service_string = format!(
                "HiddenServiceDir /var/lib/tor/app-{}\nHiddenServicePort 80 <app-{}-{}-ip>:{}\n",
                app_name_slug,
                app_name_slug,
                service_name_slug,
                main_port
            );
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
) -> Result<ResultYml, String> {
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
    let mut permissions = flatten(app.metadata.permissions.clone());

    let main_service = get_main_container(&app)?;
    let mut converted_port_map: Option<HashMap<String, Vec<PortMapElement>>> = None;
    if let Some(real_port_map) = port_map {
        let conversion_result = validate_port_map_app(real_port_map);
        match conversion_result {
            Err(conversion_error) => {
                return Err(conversion_error.to_string());
            }
            Ok(conversion_result) => {
                converted_port_map = Some(conversion_result);
            }
        }
    }
    let main_port = get_main_port(&app.services, &main_service, &converted_port_map)?;

    // Required for dynamic ports
    let env_var = format!(
        "APP_{}_{}_PORT",
        app_name.replace('-', "_").to_uppercase(),
        main_service.to_uppercase()
    );

    let replace_env_vars = HashMap::<String, String>::from([(env_var, main_port.to_string())]);

    // Copy all properties that are the same in docker-compose.yml and need no or only a simple validation
    for (service_name, service) in &app.services {
        let base_result = Service {
            image: Some(service.image.clone()),
            restart: service.restart.clone(),
            stop_grace_period: service.stop_grace_period.clone(),
            stop_signal: service.stop_signal.clone(),
            user: service.user.clone(),
            init: service.init,
            depends_on: service.depends_on.clone(),
            extra_hosts: service.extra_hosts.clone(),
            ports: Vec::new(),
            volumes: Vec::new(),
            ..Default::default()
        };
        spec_services.insert(service_name.to_string(), base_result);
        validate_service(
            app_name,
            &mut permissions,
            service,
            &replace_env_vars,
            spec_services.get_mut(service_name).unwrap(),
        )?;
    }
    // We can now finalize the process by parsing some of the remaining values
    configure_ports(&app.services, &main_service, &mut spec, &converted_port_map)?;

    let ip_address_result = define_ip_addresses(app_name, &app.services, &main_service, &mut spec);

    if ip_address_result.is_err() {
        return Err(ip_address_result.err().unwrap());
    }

    convert_volumes(&app.services, &permissions, &mut spec)?;

    let mut main_port_host: Option<u16> = None;
    if let Some(converted_map) = converted_port_map {
        main_port_host = Some(
            get_host_port(converted_map.get(&main_service).unwrap(), main_port)
                .unwrap()
                .public_port,
        );
    }

    let mut metadata = app.metadata;
    metadata.id = Some(app_name.to_string());
    metadata.permissions = permissions.iter().map(|val| Permissions::OneDependency(val.to_string())).collect();
    let result = ResultYml {
        spec,
        new_tor_entries: get_hidden_services(app_name, app.services, &main_service, main_port),
        port: main_port_host.unwrap_or(main_port),
        metadata,
    };

    // And we're done
    Ok(result)
}

#[cfg(test)]
mod test {
    use super::convert_config;
    use crate::{
        composegenerator::{
            compose::types::{ComposeSpecification, Service},
            types::{Metadata, Permissions, ResultYml},
            v4::types::{AppYml, Container},
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
        let expected_result = ResultYml {
            port: 3000,
            new_tor_entries: "HiddenServiceDir /var/lib/tor/app-example-app\nHiddenServicePort 80 <app-example-app-main-ip>:3000\n".to_string(),
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
