pub mod compose;
#[cfg(feature = "umbrel")]
pub mod umbrel;
pub mod v4;
pub mod types;

use serde_json::{Map, Value};

use self::v4::types::AppYml as AppYmlV4;
use self::types::ResultYml;

pub enum AppYmlFile {
    V4(AppYmlV4),
}

pub fn load_config<R>(app_reader: R) -> Result<AppYmlFile, String>
where
    R: std::io::Read,
{
    let app_yml = serde_yaml::from_reader::<R, serde_yaml::Value>(app_reader)
        .expect("Failed to parse app.yml");
    if !app_yml.is_mapping() {
        return Err("App.yml is not a map!".to_string());
    }
    if app_yml.get("citadel_version").is_none()
        || !app_yml.get("citadel_version").unwrap().is_number()
    {
        return Err("Citadel file format is not set or not a number!".to_string());
    }
    match app_yml.get("citadel_version").unwrap().as_u64().unwrap() {
        4 => {
            let app_definition: Result<AppYmlV4, serde_yaml::Error> = serde_yaml::from_value(app_yml);
            match app_definition {
                Ok(app_definition) => Ok(AppYmlFile::V4(app_definition)),
                Err(error) => Err(format!("Error loading app.yml as v4: {}", error)),
            }
        }
        _ => Err("Version not supported".to_string()),
    }
}

pub fn convert_config<R>(
    app_name: &str,
    app_reader: R,
    port_map: &Option<&Map<String, Value>>,
) -> Result<ResultYml, String>
where
    R: std::io::Read,
{
    let app_yml = load_config(app_reader).expect("Failed to parse app.yml");
    match app_yml {
        AppYmlFile::V4(app_definition) => {
            v4::convert::convert_config(app_name, app_definition, port_map)
        }
    }
}
