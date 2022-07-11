use serde::{Serialize, Deserialize};
use schemars::JsonSchema;
use std::collections::HashMap;

use crate::composegenerator::compose::types::ComposeSpecification;

#[derive(Serialize, Deserialize, Debug, JsonSchema, Clone, PartialEq)]
#[serde(untagged)]
pub enum Command {
    SimpleCommand(String),
    ArrayCommand(Vec<String>),
}

#[derive(Serialize, Deserialize, Debug, JsonSchema, Clone, PartialEq)]
#[serde(untagged)]
pub enum Permissions {
    OneDependency(String),
    AlternativeDependency(Vec<String>),
}


#[derive(Serialize, Deserialize, Debug, JsonSchema, Clone, PartialEq)]
#[serde(untagged)]
pub enum HiddenServices {
    PortMap(HashMap<u16, u16>),
    LayeredMap(HashMap<String, HashMap<u16, u16>>),
}

#[derive(Serialize, Deserialize, Debug, JsonSchema, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    /// The name of the app
    pub name: String,
    /// The version of the app
    pub version: String,
    /// The category for the app
    pub category: String,
    /// A short tagline for the app
    pub tagline: String,
    // Developer name -> their website
    pub developers: HashMap<String, String>,
    #[serde(default)]
    /// Permissions the app requires
    pub permissions: Vec<Permissions>,
    /// App repository name -> repo URL
    pub repo: HashMap<String, String>,
    /// A support link for the app
    pub support: String,
    /// A list of promo images for the apps
    pub gallery: Option<Vec<String>>,
    /// The path the "Open" link on the dashboard should lead to
    pub path: Option<String>,
    /// The app's default password. Can also be $APP_SEED for a random password
    pub default_password: Option<String>,
    #[serde(default = "bool::default")]
    /// True if the app only works over Tor
    pub tor_only: bool,
    /// A list of containers to update automatically (still validated by the Citadel team)
    pub update_containers: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema, Clone, PartialEq)]
pub struct PortsDefinition {
    pub tcp: Option<HashMap<u16, u16>>,
    pub udp: Option<HashMap<u16, u16>>,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema, Clone, PartialEq)]
pub enum PortPriority {
    /// Outside port doesn't matter
    Optional,
    /// Outside port is preferred, but not required for the app to work
    Recommended,
    /// Port is required for the app to work
    Required,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema, Clone, PartialEq)]
pub struct Mounts {
    pub bitcoin: Option<String>,
    pub lnd: Option<String>,
    pub c_lightning: Option<String>,
    pub data: Option<HashMap<String, String>>,
}

fn true_as_func() -> bool {
    true
}


#[derive(Serialize, Deserialize, Debug, JsonSchema, Clone, PartialEq)]
pub struct Container {
    // These can be copied directly without validation
    pub image: String,
    pub user: Option<String>,
    pub stop_grace_period: Option<String>,
    pub stop_signal: Option<String>,
    pub depends_on: Option<Vec<String>>,
    pub network_mode: Option<String>,
    pub restart: Option<String>,
    pub init: Option<bool>,
    pub extra_hosts: Option<Vec<String>>,
    // These need security checks
    pub entrypoint: Option<Command>,
    pub command: Option<Command>,
    pub environment: Option<HashMap<String, String>>,
    // These are not directly present in a compose file and need to be converted
    pub port: Option<u16>,
    // This is currently handled on the host
    pub port_priority: Option<PortPriority>,
    pub required_ports: Option<PortsDefinition>,
    pub mounts: Option<Mounts>,
    #[serde(default = "true_as_func")]
    pub enable_networking: bool,
    pub hidden_services: Option<HiddenServices>,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema, Clone, PartialEq)]
/// Citadel app definition
pub struct AppYml {
    pub citadel_version: u8,
    pub metadata: Metadata,
    pub services: HashMap<String, Container>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortMapElement {
    /// True if the port is defined by an env var and can be anything
    pub dynamic: bool,
    pub internal_port: u16,
    pub outside_port: u16,
}

#[derive(Serialize, Deserialize)]
pub struct FinalResult {
    pub port: u16,
    pub new_tor_entries: String,
    pub spec: ComposeSpecification,
}
