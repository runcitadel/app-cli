pub mod v4;
pub mod compose;
pub mod permissions;
pub mod utils;
pub mod umbrel;

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
    let app_yml = parsed.unwrap();
    if !app_yml.is_mapping() {
        return Err("App.yml is not a map!".to_string());
    }
    if app_yml.get("citadel_version").is_none() || !app_yml.get("citadel_version").unwrap().is_number() {
        return Err("Citadel file format is not set or not a number!".to_string());
    }
    match app_yml.get("citadel_version").unwrap().as_u64().unwrap() {
        4 => {
            let app_definition: Result<AppYml, serde_yaml::Error> = serde_yaml::from_str(app);
            if let Ok(app_def) = app_definition {
                v4::convert::convert_config(
                    app_name,
                    app_def,
                    port_map,
                )
            } else {
                Err(format!("Error loading app.yml as v4: {}", app_definition.err().unwrap()))
            }
        }
        _ => {
            Err("Version not supported".to_string())
        }
    }
}
