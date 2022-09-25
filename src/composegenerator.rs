pub mod compose;
pub mod types;
#[cfg(feature = "umbrel")]
pub mod umbrel;
pub mod v3;
pub mod v4;

use serde_json::{Map, Value};

use self::types::ResultYml;
use self::v3::types::Schema as AppYmlV3;
use self::v4::types::AppYml as AppYmlV4;

pub enum AppYmlFile {
    V3(AppYmlV3),
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
    let version: u64;
    if app_yml.get("citadel_version").is_none()
        || !app_yml.get("citadel_version").unwrap().is_number()
    {
        if app_yml.get("version").is_some() && app_yml.get("version").unwrap().is_number() {
            version = app_yml.get("version").unwrap().as_u64().unwrap();
        } else {
            return Err("Citadel file format is not set or not a number!".to_string());
        }
    } else {
        version = app_yml.get("citadel_version").unwrap().as_u64().unwrap();
    }
    match version {
        3 => {
            let app_definition: Result<AppYmlV3, serde_yaml::Error> =
                serde_yaml::from_value(app_yml);
            match app_definition {
                Ok(app_definition) => Ok(AppYmlFile::V3(app_definition)),
                Err(error) => Err(format!("Error loading app.yml as v3: {}", error)),
            }
        },
        4 => {
            let app_definition: Result<AppYmlV4, serde_yaml::Error> =
                serde_yaml::from_value(app_yml);
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
        AppYmlFile::V3(app_definition) => {
            v3::convert::convert_config(app_name, app_definition, port_map)
        },
    }
}
