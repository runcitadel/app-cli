use std::collections::HashMap;

use serde_json::{Map, Value};

use super::compose::types::ComposeSpecification;
use super::permissions;
use super::v4::types::Command;
use super::v4::types::PortMapElement;
use crate::utils::find_env_vars;

pub fn validate_cmd(
    app_name: &str,
    command: &Command,
    permissions: &[String],
) -> Result<(), String> {
    match command {
        Command::SimpleCommand(simple_command) => {
            let env_vars = find_env_vars(simple_command);
            for env_var in env_vars {
                if !permissions::is_allowed_by_permissions(app_name, env_var, permissions) {
                    return Err(format!("Env var {} not allowed by permissions", env_var));
                }
            }
        }
        Command::ArrayCommand(values) => {
            for value in values {
                let env_vars = find_env_vars(value);
                for env_var in env_vars {
                    if !permissions::is_allowed_by_permissions(app_name, env_var, permissions) {
                        return Err(format!("Env var {} not allowed by permissions", env_var));
                    }
                }
            }
        }
    }
    Ok(())
}

pub fn get_host_port(
    port_map: &[PortMapElement],
    internal_port: u16,
) -> Option<&PortMapElement> {
    return port_map
        .iter()
        .find(|&elem| elem.internal_port == internal_port);
}

pub fn validate_port_map_app(
    port_map_app: &Map<String, Value>,
) -> Result<HashMap<String, Vec<PortMapElement>>, String> {
    let mut result: HashMap<String, Vec<PortMapElement>> =
        HashMap::<String, Vec<PortMapElement>>::new();

    for container_name in port_map_app.keys() {
        let container = port_map_app.get(container_name).unwrap();
        result.insert(container_name.to_string(), Vec::<PortMapElement>::new());
        let container_result = result.get_mut(container_name).unwrap();
        if !container.is_array() {
            return Err("Container invalid in port map".to_string());
        }

        for element in container.as_array().unwrap() {
            if !element.is_object()
                || element.as_object().unwrap().get("publicPort").is_none()
                || element.as_object().unwrap().get("internalPort").is_none()
                || element.as_object().unwrap().get("dynamic").is_none()
                || !element
                    .as_object()
                    .unwrap()
                    .get("publicPort")
                    .unwrap()
                    .is_i64()
                || !element
                    .as_object()
                    .unwrap()
                    .get("internalPort")
                    .unwrap()
                    .is_i64()
                || !element
                    .as_object()
                    .unwrap()
                    .get("dynamic")
                    .unwrap()
                    .is_boolean()
            {
                return Err("Container invalid in port map".to_string());
            }

            let elem = element.as_object().unwrap();
            let entry = PortMapElement {
                dynamic: elem.get("dynamic").unwrap().as_bool().unwrap(),
                internal_port: elem.get("internalPort").unwrap().as_i64().unwrap() as u16,
                outside_port: elem.get("publicPort").unwrap().as_i64().unwrap() as u16,
            };

            container_result.push(entry);
        }
    }

    Ok(result)
}

pub fn get_main_container(spec: &ComposeSpecification) -> Result<String, String> {
    let mut main_service_name: Option<String> = Option::<String>::None;
    // We now have a list of services whose dependencies are present
    // And that are mostly validated
    // We can now determine the main container of the app
    for service_name in spec.services.as_ref().unwrap().keys() {
        // web is for easier porting from Umbrel
        if service_name == "main" || service_name == "web" {
            main_service_name = Some(service_name.to_string());
            break;
        } else if service_name.starts_with("main") {
            if main_service_name.is_some() {
                log::info!(
                    "Container {} and {} could both be main container",
                    service_name,
                    main_service_name.unwrap()
                );
                return Err("Multiple main containers in app!".to_string());
            }
            main_service_name = Some(service_name.to_string());
        }
    }
    if let Some(main_name) = main_service_name {
        Ok(main_name)
    } else {
        Err("No main container found!".to_string())
    }
}
