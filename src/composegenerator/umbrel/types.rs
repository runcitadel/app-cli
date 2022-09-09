use serde::{Serialize, Deserialize};
#[cfg(feature = "schema")]
use schemars::JsonSchema;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    /// The version of the metadata file
    pub manifest_version: i8,
    /// The app id
    pub id: String,
    /// The name of the app
    pub name: String,
    /// The version of the app
    pub version: String,
    /// The category for the app
    pub category: String,
    /// A short tagline for the app
    pub tagline: String,
    // Developer name
    pub developer: String,
    // Developer wbsite
    pub website: String,
    #[serde(default)]
    /// Permissions the app requires
    pub dependencies: Vec<String>,
    /// App repository name -> repo URL
    pub repo: String,
    /// A support link for the app
    pub support: String,
    /// A list of promo images for the apps
    pub gallery: Option<Vec<String>>,
    /// The path the "Open" link on the dashboard should lead to
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// The app's default password. Can also be $APP_SEED for a random password
    pub default_password: Option<String>,
    #[serde(default = "bool::default")]
    /// True if the app only works over Tor
    pub tor_only: bool,
    /// The apps port
    pub port: u16,
    #[serde(default = "bool::default")]
    pub deterministic_password: bool,
    /// A description of the app
    pub description: String,
}