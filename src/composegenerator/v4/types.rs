#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::composegenerator::compose::types::{Command, StringOrInt};
use crate::composegenerator::types::Metadata;


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(untagged)]
pub enum HiddenServices {
    PortMap(HashMap<u16, u16>),
    LayeredMap(HashMap<String, HashMap<u16, u16>>),
}



#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct PortsDefinition {
    pub tcp: Option<HashMap<u16, u16>>,
    pub udp: Option<HashMap<u16, u16>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub enum PortPriority {
    /// Outside port doesn't matter
    Optional,
    /// Outside port is preferred, but not required for the app to work
    Recommended,
    /// Port is required for the app to work
    Required,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct Mounts {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bitcoin: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lnd: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub c_lightning: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<HashMap<String, String>>,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct Container {
    // These can be copied directly without validation
    pub image: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_grace_period: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_signal: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depends_on: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restart: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub init: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_hosts: Option<Vec<String>>,
    // These need security checks
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entrypoint: Option<Command>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<Command>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<HashMap<String, StringOrInt>>,
    // These are not directly present in a compose file and need to be converted
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    // This is currently handled on the host
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port_priority: Option<PortPriority>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_ports: Option<PortsDefinition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mounts: Option<Mounts>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_networking: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hidden_services: Option<HiddenServices>,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
/// Citadel app definition
pub struct AppYml {
    pub citadel_version: u8,
    pub metadata: Metadata,
    pub services: HashMap<String, Container>,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PortMapElement {
    /// True if the port is defined by an env var and can be anything
    pub dynamic: bool,
    pub internal_port: u16,
    pub public_port: u16,
}
