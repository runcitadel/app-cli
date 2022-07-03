use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::composegenerator::v4::types::Command;

#[derive(Clone, PartialEq, Debug, Default, Deserialize, Serialize)]
#[serde(rename = "blkio_limit")]
pub struct BlkioLimit {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate: Option<serde_yaml::Value>,
}
#[derive(Clone, PartialEq, Debug, Default, Deserialize, Serialize)]
#[serde(rename = "blkio_weight")]
pub struct BlkioWeight {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<i64>,
}
#[derive(Clone, PartialEq, Debug, Default, Deserialize, Serialize)]
#[serde(rename = "config")]
pub struct Config {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external: Option<serde_yaml::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<ListOrDict>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_driver: Option<String>,
}
pub type Constraints = serde_yaml::Value;
#[derive(Clone, PartialEq, Debug, Default, Deserialize, Serialize)]
pub struct DeploymentPlacementItemItemPreferences {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spread: Option<String>,
}
#[derive(Clone, PartialEq, Debug, Default, Deserialize, Serialize)]
pub struct DeploymentPlacement {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constraints: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_replicas_per_node: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferences: Option<Vec<DeploymentPlacementItemItemPreferences>>,
}
#[derive(Clone, PartialEq, Debug, Default, Deserialize, Serialize)]
pub struct DeploymentPlacementItemItemPreferencesResourcesLimits {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpus: Option<serde_yaml::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pids: Option<i64>,
}
#[derive(Clone, PartialEq, Debug, Default, Deserialize, Serialize)]
pub struct DeploymentPlacementItemItemPreferencesResourcesLimitsReservations {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpus: Option<serde_yaml::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub devices: Option<Devices>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generic_resources: Option<GenericResources>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory: Option<String>,
}
#[derive(Clone, PartialEq, Debug, Default, Deserialize, Serialize)]
pub struct DeploymentPlacementItemItemPreferencesResources {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limits: Option<DeploymentPlacementItemItemPreferencesResourcesLimits>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reservations: Option<DeploymentPlacementItemItemPreferencesResourcesLimitsReservations>,
}
#[derive(Clone, PartialEq, Debug, Default, Deserialize, Serialize)]
pub struct DeploymentPlacementItemItemPreferencesResourcesLimitsReservationsRestartPolicy {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delay: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_attempts: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window: Option<String>,
}
#[derive(Clone, PartialEq, Debug, Default, Deserialize, Serialize)]
pub struct DeploymentPlacementItemItemPreferencesResourcesLimitsReservationsRestartPolicyRollbackConfig
{
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delay: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_action: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_failure_ratio: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monitor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallelism: Option<i64>,
}
#[derive(Clone, PartialEq, Debug, Default, Deserialize, Serialize)]
pub struct DeploymentPlacementItemItemPreferencesResourcesLimitsReservationsRestartPolicyRollbackConfigUpdateConfig
{
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delay: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_action: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_failure_ratio: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monitor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallelism: Option<i64>,
}
#[derive(Clone, PartialEq, Debug, Default, Deserialize, Serialize)]
#[serde(rename = "deployment")]
pub struct Deployment { # [serde (skip_serializing_if = "Option::is_none")] pub endpoint_mode : Option < String > , # [serde (skip_serializing_if = "Option::is_none")] pub labels : Option < ListOrDict > , # [serde (skip_serializing_if = "Option::is_none")] pub mode : Option < String > , # [serde (skip_serializing_if = "Option::is_none")] pub placement : Option < DeploymentPlacement > , # [serde (skip_serializing_if = "Option::is_none")] pub replicas : Option < i64 > , # [serde (skip_serializing_if = "Option::is_none")] pub resources : Option < DeploymentPlacementItemItemPreferencesResources > , # [serde (skip_serializing_if = "Option::is_none")] pub restart_policy : Option < DeploymentPlacementItemItemPreferencesResourcesLimitsReservationsRestartPolicy > , # [serde (skip_serializing_if = "Option::is_none")] pub rollback_config : Option < DeploymentPlacementItemItemPreferencesResourcesLimitsReservationsRestartPolicyRollbackConfig > , # [serde (skip_serializing_if = "Option::is_none")] pub update_config : Option < DeploymentPlacementItemItemPreferencesResourcesLimitsReservationsRestartPolicyRollbackConfigUpdateConfig > }
#[derive(Clone, PartialEq, Debug, Default, Deserialize, Serialize)]
pub struct DevicesItemParallelism {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capabilities: Option<ListOfStrings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<serde_yaml::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_ids: Option<ListOfStrings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub driver: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<ListOrDict>,
}
pub type Devices = Vec<DevicesItemParallelism>;
#[derive(Clone, PartialEq, Debug, Default, Deserialize, Serialize)]
pub struct GenericResourcesItemOptionsDiscreteResourceSpec {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<f64>,
}
#[derive(Clone, PartialEq, Debug, Default, Deserialize, Serialize)]
pub struct GenericResourcesItemOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discrete_resource_spec: Option<GenericResourcesItemOptionsDiscreteResourceSpec>,
}
pub type GenericResources = Vec<GenericResourcesItemOptions>;
#[derive(Clone, PartialEq, Debug, Default, Deserialize, Serialize)]
#[serde(rename = "healthcheck")]
pub struct Healthcheck {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interval: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retries: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_period: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test: Option<serde_yaml::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<String>,
}
pub type ListOfStrings = Vec<String>;
pub type ListOrDict = serde_yaml::Value;
#[derive(Clone, PartialEq, Debug, Default, Deserialize, Serialize)]
pub struct NetworkIpamItemConfigAuxAddresses {}
#[derive(Clone, PartialEq, Debug, Default, Deserialize, Serialize)]
pub struct NetworkIpamItemConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aux_addresses: Option<NetworkIpamItemConfigAuxAddresses>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gateway: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_range: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subnet: Option<String>,
}
#[derive(Clone, PartialEq, Debug, Default, Deserialize, Serialize)]
pub struct NetworkIpamItemConfigAuxAddressesOptions {}
#[derive(Clone, PartialEq, Debug, Default, Deserialize, Serialize)]
pub struct NetworkIpam {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<Vec<NetworkIpamItemConfig>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub driver: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<NetworkIpamItemConfigAuxAddressesOptions>,
}
#[derive(Clone, PartialEq, Debug, Default, Deserialize, Serialize)]
#[serde(rename = "network")]
pub struct Network {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub driver: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub driver_opts: Option<::std::collections::BTreeMap<String, serde_yaml::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "enable_ipv6")]
    pub enable_ipv_6: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external: Option<serde_yaml::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub internal: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipam: Option<NetworkIpam>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<ListOrDict>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}
#[derive(Clone, PartialEq, Debug, Default, Deserialize, Serialize)]
#[serde(rename = "secret")]
pub struct Secret {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub driver: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub driver_opts: Option<::std::collections::BTreeMap<String, serde_yaml::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external: Option<serde_yaml::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<ListOrDict>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_driver: Option<String>,
}
#[derive(Clone, PartialEq, Debug, Default, Deserialize, Serialize)]
pub struct ServiceBlkioConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_read_bps: Option<Vec<BlkioLimit>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_read_iops: Option<Vec<BlkioLimit>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_write_bps: Option<Vec<BlkioLimit>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_write_iops: Option<Vec<BlkioLimit>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight_device: Option<Vec<BlkioWeight>>,
}
#[derive(Clone, PartialEq, Debug, Default, Deserialize, Serialize)]
pub struct ServiceBlkioConfigItemItemItemItemItemItemItemCredentialSpec {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registry: Option<String>,
}
#[derive(Clone, PartialEq, Debug, Default, Deserialize, Serialize)]
pub struct ServiceBlkioConfigItemItemItemItemItemItemItemCredentialSpecItemItemItemItemItemItemLogging
{
    #[serde(skip_serializing_if = "Option::is_none")]
    pub driver: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<::std::collections::BTreeMap<String, serde_yaml::Value>>,
}
#[derive(Clone, PartialEq, Debug, Default, Deserialize, Serialize)]
#[serde(rename = "service")]
pub struct Service {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blkio_config: Option<ServiceBlkioConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build: Option<serde_yaml::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cap_add: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cap_drop: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cgroup_parent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<Command>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configs: Option<ServiceConfigOrSecret>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu_count: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu_percent: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu_period: Option<serde_yaml::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu_quota: Option<serde_yaml::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu_rt_period: Option<serde_yaml::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu_rt_runtime: Option<serde_yaml::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu_shares: Option<serde_yaml::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpus: Option<serde_yaml::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpuset: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential_spec: Option<ServiceBlkioConfigItemItemItemItemItemItemItemCredentialSpec>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub depends_on: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deploy: Option<Deployment>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_cgroup_rules: Option<ListOfStrings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub devices: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dns: Option<StringOrList>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dns_opt: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dns_search: Option<StringOrList>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domainname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entrypoint: Option<Command>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env_file: Option<StringOrList>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<HashMap::<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expose: Option<Vec<serde_yaml::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extends: Option<serde_yaml::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_links: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_hosts: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_add: Option<Vec<serde_yaml::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub healthcheck: Option<Healthcheck>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub init: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipc: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub isolation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<ListOrDict>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logging: Option<
        ServiceBlkioConfigItemItemItemItemItemItemItemCredentialSpecItemItemItemItemItemItemLogging,
    >,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mac_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mem_limit: Option<serde_yaml::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mem_reservation: Option<serde_yaml::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mem_swappiness: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memswap_limit: Option<serde_yaml::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub networks: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oom_kill_disable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oom_score_adj: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pids_limit: Option<serde_yaml::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub ports: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub privileged: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profiles: Option<ListOfStrings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pull_policy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_only: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restart: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secrets: Option<ServiceConfigOrSecret>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_opt: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shm_size: Option<serde_yaml::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stdin_open: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_grace_period: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_signal: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage_opt: Option<::std::collections::BTreeMap<String, serde_yaml::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sysctls: Option<ListOrDict>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tmpfs: Option<StringOrList>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tty: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ulimits: Option<::std::collections::BTreeMap<String, serde_yaml::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub userns_mode: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub volumes: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volumes_from: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_dir: Option<String>,
}
pub type ServiceConfigOrSecret = Vec<serde_yaml::Value>;
pub type StringOrList = serde_yaml::Value;
#[derive(Clone, PartialEq, Debug, Default, Deserialize, Serialize)]
#[serde(rename = "volume")]
pub struct Volume {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub driver: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub driver_opts: Option<::std::collections::BTreeMap<String, serde_yaml::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external: Option<serde_yaml::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<ListOrDict>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}
#[derive(Clone, PartialEq, Debug, Default, Deserialize, Serialize)]
pub struct ComposeSpecificationConfigs {}
#[derive(Clone, PartialEq, Debug, Default, Deserialize, Serialize)]
pub struct ComposeSpecificationConfigsSecrets {}
#[derive(Clone, PartialEq, Debug, Default, Deserialize, Serialize)]
pub struct ComposeSpecificationConfigsSecretsServices {}
#[derive(Clone, PartialEq, Debug, Default, Deserialize, Serialize)]
pub struct ComposeSpecificationConfigsSecretsServicesVolumes {}
#[derive(Clone, PartialEq, Debug, Default, Deserialize, Serialize)]
#[serde(rename = "Compose Specification")]
pub struct ComposeSpecification {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configs: Option<ComposeSpecificationConfigs>,
    #[doc = " define the Compose project name, until user defines one explicitly."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub networks: Option<::std::collections::BTreeMap<String, serde_yaml::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secrets: Option<ComposeSpecificationConfigsSecrets>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub services: Option<HashMap<String, Service>>,
    #[doc = " declared for backward compatibility, ignored."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volumes: Option<ComposeSpecificationConfigsSecretsServicesVolumes>,
}
