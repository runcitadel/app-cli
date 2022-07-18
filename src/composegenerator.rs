pub mod v4;
pub mod compose;
pub mod permissions;
pub mod utils;

use serde_json::{Map, Value};

use crate::composegenerator::v4::types::{AppYml, FinalResult};

pub fn convert_config(
    app_name: &str,
    app: &str,
    port_map: &Map<String, Value>,
) -> Result<FinalResult, String> {
    let parsed = serde_yaml::from_str::<serde_yaml::Value>(app);
    if parsed.is_err() {
        return Err(format!("Failed to parse app.yml: {:#?}", parsed.err().unwrap()))
    }
    let appYml = parsed.unwrap();
    if !appYml.is_mapping() {
        return Err("App.yml is not a map!".to_string());
    }
    if !appYml.get("citadel_version").is_some() || !appYml.get("citadel_version").unwrap().is_number() {
        return Err("Citadel file format is not set or not a number!".to_string());
    }
    match appYml.get("citadel_version").unwrap().as_u64().unwrap() {
        4 => {
            let app_definition: Result<AppYml, serde_yaml::Error> = serde_yaml::from_str(app);
            if app_definition.is_err() {
                return Err(format!("Error loading app.yml as v4: {}", app_definition.err().unwrap()));
            } else {
                return v4::convert::convert_config(
                    app_name,
                    app_definition.unwrap(),
                    port_map,
                );
            }
        }
        _ => {
            return Err("Version not supported".to_string());
        }
    }
}
