use std::collections::HashMap;

#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::composegenerator::{
    compose::types::{Command, StringOrIntOrBool},
    types::Permissions,
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(untagged)]
pub enum HiddenServices {
    PortMap(HashMap<u16, u16>),
    LayeredMap(HashMap<String, HashMap<u16, u16>>),
    LegacyLayeredMap(HashMap<String, Vec<u16>>),
    LegacySinglePort(u16),
    LegacyPortArray(Vec<u16>),
    LegacyMap(HashMap<String, u16>),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(untagged)]
pub enum RepoDefinition {
    RepoUrl(String),
    MultiRepo(HashMap<String, String>),
}

#[derive(Clone, PartialEq, Eq, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct SchemaItemContainersMounts {
    /// Where to mount the bitcoin dir
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bitcoin: Option<String>,
    /// Where to mount the c-lightning dir
    #[serde(skip_serializing_if = "Option::is_none")]
    pub c_lightning: Option<String>,
    /// Where to mount the lnd dir
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lnd: Option<String>,
}
#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct SchemaItemContainers {
    pub name: String,
    pub image: String,
    /// The command for the container
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<Command>,
    /// An array of at directories in the container the app stores its data in. Can be empty.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Vec<String>>,
    /// The services the container depends on
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depends_on: Option<Vec<String>>,
    /// The entrypoint for the container
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entrypoint: Option<Command>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<HashMap<String, StringOrIntOrBool>>,
    /// This can either be a map of hidden service names (human readable names, not the .onion URL,
    /// and strings, not numbers) to a port if your app needs multiple hidden services on different
    /// ports, a map of port inside to port on the hidden service (if your app has multiple ports
    /// on one hidden service), or simply one port number if your apps hidden service should only
    /// expose one port to the outside which isn't 80.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "hiddenServicePorts")]
    pub hidden_service_ports: Option<HiddenServices>,
    /// Whether the container should be run with init
    #[serde(skip_serializing_if = "Option::is_none")]
    pub init: Option<bool>,
    /// Where to mount some services' data directories
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mounts: Option<SchemaItemContainersMounts>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_mode: Option<String>,
    /// Set this to true if the container shouldn't get an IP & port exposed. This isn't necessary,
    /// but helps the docker-compose.yml generator to generate a cleaner output.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "noNetwork")]
    pub no_network: Option<bool>,
    /// If this is the main container, the port inside the container which will be exposed to the
    /// outside as the port specified in metadata. If this is not set, the port is passed as an env
    /// variable in the format APP_${APP_NAME}_${CONTAINER_NAME}_PORT
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    /// The port this container would like to have \"port\" exposed as.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "preferredOutsidePort")]
    pub preferred_outside_port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "requiredPorts")]
    pub required_ports: Option<Vec<u16>>,
    /// Ports this container requires to be exposed to work properly
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "requiredUdpPorts")]
    pub required_udp_ports: Option<Vec<u16>>,
    /// Dependencies this container requires, it is ignored without it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires: Option<Vec<String>>,
    /// Set this to true if the app requires the preferredOutsidePort to be the real outside port.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "requiresPort")]
    pub requires_port: Option<bool>,
    /// When the container should restart. Can be 'always' or 'on-failure'.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restart: Option<String>,
    /// The grace period for stopping the container. Defaults to 1 minute.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_grace_period: Option<String>,
    /// The signal to send to the container when stopping
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_signal: Option<String>,
    /// The user the container should run as
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}
#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct SchemaMetadata {
    /// The category you'd put the app in
    pub category: String,
    /// Displayed name of the app
    pub name: String,
    /// Displayed version for the app
    pub version: String,
    /// A clever tagline
    pub tagline: String,
    /// A longer description of the app
    pub description: String,
    /// The awesome people behind the app
    pub developers: HashMap<String, String>,
    /// The services the app depends on.
    /// This can also contain an array like [c-lightning, lnd] if your app requires one of two
    /// dependencies to function.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependencies: Option<Vec<Permissions>>,
    /// The development repository (or repositories) for your app, if you have multiple, in the
    /// format human readable name: repo url
    pub repo: RepoDefinition,
    /// A link to the app support wiki/chat/...
    pub support: String,
    /// URLs or paths in the runcitadel/app-images/[app-name] folder with app images
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gallery: Option<Vec<String>>,
    /// The path of the app's visible site the open button should open
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Whether the app is only available over tor
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "torOnly")]
    pub tor_only: Option<bool>,
    /// The app's default password. Set this to $APP_SEED if the password is the environment
    /// variable $APP_SEED.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "defaultPassword")]
    pub default_password: Option<String>,
}
#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Schema {
    /// The version of the app.yml format you're using.
    pub version: u64,
    pub metadata: SchemaMetadata,
    pub containers: Vec<SchemaItemContainers>,
}
