use citadel_apps::composegenerator::convert_config;
#[cfg(any(feature = "dev-tools", feature = "preprocess"))]
use citadel_apps::composegenerator::load_config;
#[cfg(feature = "dev-tools")]
use citadel_apps::{composegenerator::v4::types::AppYml, updates::update_app};
#[cfg(feature = "preprocess")]
use citadel_apps::{
    composegenerator::v4::{permissions::is_allowed_by_permissions, utils::derive_entropy},
    utils::flatten,
};
use clap::{Parser, Subcommand};
#[cfg(feature = "preprocess")]
use std::io::{Read, Write};
#[cfg(any(feature = "umbrel", feature = "preprocess"))]
use std::path::Path;
#[cfg(any(feature = "dev-tools", feature = "preprocess"))]
use std::process::exit;
#[cfg(feature = "preprocess")]
use tera::{Context, Tera};

#[derive(Subcommand, Debug)]
enum SubCommand {
    /// Convert a citadel app.yml to a result.yml file
    Convert {
        /// The app file to run this on
        app: String,
        /// The app's ID
        #[clap(short, long)]
        app_name: String,
        /// The output file to save the result to
        output: String,
        /// The port map file
        #[clap(short, long)]
        port_map: String,
    },
    /// Get a JSON schema for the app.yml format
    #[cfg(feature = "dev-tools")]
    Schema {
        /// The version of the app.yml format to get the schema for
        /// (defaults to 4)
        #[clap(short, long, default_value = "4")]
        version: u8,
    },
    /// Preprocess a citadel app.yml.jinja file by parsing the Tera (jinja-like) template and writing the result to a file
    /// The YAML is not validated or parsed in any way.
    #[cfg(feature = "preprocess")]
    Preprocess {
        /// The app file to run this on
        app: String,
        /// The app's ID
        #[clap(short, long)]
        app_name: String,
        /// The output file to save the result to
        output: String,
        /// The services that are installed as a list of comma separated values
        #[clap(short, long)]
        services: Option<String>,
    },
    /// Preprocess a directory by looping through its subdirectories and preprocessing each app.yml.jinja file
    /// and saving the result to an app.yml file
    #[cfg(feature = "preprocess")]
    PreprocessDir {
        /// The directory to run this on
        dir: String,
        /// The services that are installed as a list of comma separated values
        #[clap(short, long)]
        services: Option<String>,
    },
    /// Preprocess an app's config.*.jinja file by parsing the Tera (jinja-like) template and writing the result to a file
    #[cfg(feature = "preprocess")]
    PreprocessConfigFile {
        /// The config file to run this on
        config_file: String,
        /// The env file to get env vars from
        #[clap(short, long)]
        env_file: String,
        /// The app file to run this on
        #[clap(long)]
        app_file: String,
        /// The app's ID
        #[clap(long)]
        app_name: String,
        /// The output file to save the result to
        output: String,
        /// The citadel seed file
        #[clap(long)]
        seed_file: Option<String>,
        /// The services that are installed as a list of comma separated values
        #[clap(long)]
        services: Option<String>,
    },
    /// Convert an Umbrel app (by app directory path) to a Citadel app.yml file
    /// Manual fixes may be required to make the app.yml work
    #[cfg(feature = "umbrel")]
    UmbrelToCitadel {
        /// The app directory to run this on
        app: String,
        /// The output file to save the result to
        output: String,
    },
    /// Validate a Citadel app.yml file and check if it could be parsed & converted
    #[cfg(feature = "dev-tools")]
    Validate {
        /// The app file to run this on
        app: String,
        /// The app's ID
        #[clap(short, long)]
        app_name: String,
    },
    /// Update the app inside an app.yml to its latest version
    #[cfg(feature = "dev-tools")]
    Update {
        /// The app file to run this on
        app: String,
        /// A GitHub token
        #[clap(short, long)]
        gh_token: Option<String>,
        /// Whether to include pre-releases
        #[clap(short, long)]
        include_prerelease: bool,
    },
}

/// Manage apps on Citadel
#[derive(Parser)]
struct Cli {
    /// The subcommand to run
    #[clap(subcommand)]
    command: SubCommand,
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let args: Cli = Cli::parse();
    match args.command {
        SubCommand::Convert {
            app,
            app_name,
            output,
            port_map,
        } => {
            let app_yml = std::fs::File::open(app.as_str()).expect("Error opening app definition!");
            let port_map = std::fs::File::open(port_map.as_str()).expect("Error opening port map!");
            let port_map: serde_json::Map<String, serde_json::Value> =
                serde_json::from_reader(port_map).expect("Error loading port map!");
            let port_map_entry = port_map.get(&app_name).expect("App not found in port map!");
            let port_map = port_map_entry
                .as_object()
                .expect("App definition in port map is invalid!");
            let result = convert_config(&app_name, &app_yml, &Some(port_map))
                .expect("Failed to convert config!");
            let writer = std::fs::File::create(output.as_str()).unwrap();
            serde_yaml::to_writer(writer, &result).expect("Failed to save");
        }
        #[cfg(feature = "dev-tools")]
        SubCommand::Schema { version } => match version {
            4 => {
                let schema = schemars::schema_for!(AppYml);
                println!("{}", serde_yaml::to_string(&schema).unwrap());
            }
            _ => {
                log::error!("Unsupported schema version!");
                exit(1);
            }
        },
        #[cfg(feature = "preprocess")]
        SubCommand::PreprocessDir { dir, services } => {
            // Loop through the subdirectories of the directory and convert the app.yml.jinja files to app.yml
            // The app name is the name of the subdirectory
            let services = services.unwrap_or_default();
            let service_list: Vec<&str> = services.split(',').collect();
            let dir_path = Path::new(dir.as_str());
            if !dir_path.is_dir() {
                log::error!("Directory not found!");
                exit(1);
            }
            let dir_entries = dir_path.read_dir().unwrap();
            for entry in dir_entries {
                let entry = entry.unwrap();
                if entry.file_type().unwrap().is_dir() {
                    let file_name = entry.file_name();
                    let app_name = file_name.to_str().unwrap();
                    let app_file = entry.path().join("app.yml.jinja");
                    if !app_file.is_file() {
                        continue;
                    }
                    let app_definition = std::fs::read_to_string(app_file.as_path()).unwrap();
                    let mut context = Context::new();
                    context.insert("services", &service_list);
                    context.insert("app_name", &app_name);
                    let result = Tera::one_off(app_definition.as_str(), &context, false)
                        .expect("Failed to preprocess");
                    let writer = std::fs::File::create(entry.path().join("app.yml")).unwrap();
                    serde_yaml::to_writer(writer, &result).expect("Failed to save!");
                }
            }
        }
        #[cfg(feature = "preprocess")]
        SubCommand::Preprocess {
            app,
            app_name,
            output,
            services,
        } => {
            let services = services.unwrap_or_default();
            let service_list: Vec<&str> = services.split(',').collect();
            let mut app_yml =
                std::fs::File::open(app.as_str()).expect("Error opening app definition!");
            let mut context = Context::new();
            context.insert("services", &service_list);
            context.insert("app_name", &app_name);
            let mut tmpl = String::new();
            app_yml
                .read_to_string(&mut tmpl)
                .expect("Error running templating engine on app definition!");
            let tmpl_result = Tera::one_off(tmpl.as_str(), &context, false)
                .expect("Error running templating engine on app definition!");
            let mut writer = std::fs::File::create(output.as_str()).unwrap();
            writer
                .write_all(tmpl_result.as_bytes())
                .expect("Error saving file!");
        }
        #[cfg(feature = "preprocess")]
        SubCommand::PreprocessConfigFile {
            config_file,
            env_file,
            app_file,
            app_name,
            seed_file,
            output,
            services,
        } => {
            let services = services.unwrap_or_default();
            let service_list: Vec<&str> = services.split(',').collect();
            #[allow(deprecated)]
            let env_vars = dotenv::from_filename_iter(env_file).expect("Failed to load .env");
            let app_yml =
                std::fs::File::open(app_file.as_str()).expect("Error opening app definition!");
            let mut config_file =
                std::fs::File::open(config_file.as_str()).expect("Error opening config file!");
            let mut context = Context::new();
            context.insert("services", &service_list);
            context.insert("app_name", &app_name);
            let parsed_app_yml = load_config(app_yml).expect("Failed to parse app.yml");
            match parsed_app_yml {
                citadel_apps::composegenerator::AppYmlFile::V4(app_yml) => {
                    let permissions = flatten(app_yml.metadata.permissions.clone());
                    let app_id = app_name.as_str();
                    for item in env_vars {
                        let (key, val) = item.expect("Env var invalid");
                        if is_allowed_by_permissions(app_id, &key, &permissions) {
                            context.insert(key, &val);
                        }
                    }
                    if let Some(seed_path) = seed_file {
                        let mut seed_file = std::fs::File::open(seed_path.as_str())
                            .expect("Error opening seed file!");
                        let mut seed_string = String::new();
                        seed_file
                            .read_to_string(&mut seed_string)
                            .expect("Error reading seed file!");
                        context.insert(
                            "APP_SEED",
                            &derive_entropy(&seed_string, format!("app-{}-seed", app_id).as_str()),
                        );
                        for i in 1..6 {
                            context.insert(
                                format!("APP_SEED_{}", i),
                                &derive_entropy(
                                    &seed_string,
                                    format!("app-{}-seed{}", app_id, i).as_str(),
                                ),
                            );
                        }
                    }
                    context.insert("APP_VERSION", &app_yml.metadata.version);
                }
            };
            let mut tmpl = String::new();
            config_file
                .read_to_string(&mut tmpl)
                .expect("Failed to load the config file!");
            let tmpl_result = Tera::one_off(tmpl.as_str(), &context, false)
                .expect("Error running templating engine on app definition!");
            let mut writer = std::fs::File::create(output.as_str()).unwrap();
            writer
                .write_all(tmpl_result.as_bytes())
                .expect("Failed to save file");
        }
        #[cfg(feature = "umbrel")]
        SubCommand::UmbrelToCitadel { app, output } => {
            let app_dir = Path::new(&app);
            let compose_yml = std::fs::File::open(app_dir.join("docker-compose.yml"))
                .expect("Error opening docker-compose.yml!");
            let app_yml = std::fs::File::open(app_dir.join("umbrel-app.yml"))
                .expect("Error opening umbrel-app.yml!");
            let app_yml_parsed: citadel_apps::composegenerator::umbrel::types::Metadata =
                serde_yaml::from_reader(app_yml).expect("Error parsing umbrel-app.yml!");
            let compose_yml_parsed: citadel_apps::composegenerator::compose::types::ComposeSpecification
             = serde_yaml::from_reader(compose_yml).expect("Error parsing docker-compose.yml!");
            let result = citadel_apps::composegenerator::umbrel::convert::convert_compose(
                compose_yml_parsed,
                app_yml_parsed,
            );
            let writer = std::fs::File::create(output).expect("Error creating output file");
            serde_yaml::to_writer(writer, &result).expect("Error saving file!");
        }
        #[cfg(feature = "dev-tools")]
        SubCommand::Validate { app, app_name } => {
            let app_yml = std::fs::File::open(app).expect("Error opening app definition!");
            convert_config(&app_name, &app_yml, &None).expect("App is invalid");
            log::info!("App is valid!");
        }
        #[cfg(feature = "dev-tools")]
        SubCommand::Update {
            app,
            gh_token,
            include_prerelease,
        } => {
            if let Some(gh_token) = gh_token {
                octocrab::initialise(octocrab::OctocrabBuilder::new().personal_token(gh_token))
                    .expect("Failed to initialise octocrab");
            }
            let app_yml = std::fs::File::open(app.clone()).expect("Error opening app definition!");
            let mut parsed_app_yml = load_config(app_yml).expect("Failed to parse app.yml");
            update_app(&mut parsed_app_yml, include_prerelease).await;
            match parsed_app_yml {
                citadel_apps::composegenerator::AppYmlFile::V4(app_yml) => {
                    let writer = std::fs::File::create(app).expect("Error opening app definition!");
                    serde_yaml::to_writer(writer, &app_yml).expect("Error saving app definition!");
                }
            }
        }
    }
}
