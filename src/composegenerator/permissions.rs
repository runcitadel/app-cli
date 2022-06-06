pub const BITCOIN_ENV_VARS: [&str; 10] = [
    "BITCOIN_IP",
    "BITCOIN_P2P_PORT",
    "BITCOIN_RPC_PORT",
    "BITCOIN_RPC_USER",
    "BITCOIN_RPC_PASS",
    "BITCOIN_RPC_AUTH",
    "BITCOIN_ZMQ_RAWBLOCK_PORT",
    "BITCOIN_ZMQ_RAWTX_PORT",
    "BITCOIN_ZMQ_HASHBLOCK_PORT",
    "BITCOIN_ZMQ_SEQUENCE_PORT",
];

pub const LND_ENV_VARS: [&str; 3] = ["LND_IP", "LND_GRPC_PORT", "LND_REST_PORT"];

pub const ELECTRUM_ENV_VARS: [&str; 2] = ["ELECTRUM_IP", "ELECTRUM_PORT"];

pub const C_LIGHTNING_ENV_VARS: [&str; 1] = ["C_LIGHTNING_IP"];

pub const ALWAYS_ALLOWED_ENV_VARS: [&str; 10] = [
    "TOR_PROXY_IP",
    "TOR_PROXY_PORT",
    "APP_DOMAIN",
    "APP_HIDDEN_SERVICE",
    "BITCOIN_NETWORK",
    "APP_SEED",
    "APP_SEED_2",
    "APP_SEED_3",
    "APP_SEED_4",
    "APP_SEED_5",
];

pub fn is_allowed_by_permissions(app_id: &str, env_var: &str, permissions: &[String]) -> bool {
    if ALWAYS_ALLOWED_ENV_VARS.contains(&env_var) {
        return true;
    } else if env_var.starts_with("BITCOIN") {
        return permissions.contains(&"bitcoind".to_string())
            && BITCOIN_ENV_VARS.contains(&env_var);
    } else if env_var.starts_with("LND") {
        return permissions.contains(&"lnd".to_string())
            && LND_ENV_VARS.contains(&env_var);
    } else if env_var.starts_with("ELECTRUM") {
        return permissions.contains(&"electrum".to_string())
            && ELECTRUM_ENV_VARS.contains(&env_var);
    } else if env_var.starts_with("C_LIGHTNING") {
        return permissions.contains(&"c-lightning".to_string())
            && C_LIGHTNING_ENV_VARS.contains(&env_var);
    } else if env_var.starts_with("APP_HIDDEN_SERVICE_") || env_var.starts_with("APP_SEED") {
        return true;
    } else if env_var.starts_with("APP_") {
        // Remove the APP_
        let mut app_name: &str = env_var.split_once('_').unwrap().1;
        // Remove the _IP / _PORT / _SHAREDSECRET
        app_name = app_name.rsplit_once('_').unwrap().0;
        // Remove the container name
        app_name = app_name.rsplit_once('_').unwrap().0;
        let lower_app_name: String = app_name.to_lowercase();
        let app_permission_name = lower_app_name.as_str().replace('_', "-");
        return app_id == app_permission_name || permissions.contains(&app_permission_name);
    }
    false
}
