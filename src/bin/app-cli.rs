#[cfg(feature = "dev-tools")]
use citadel_apps::composegenerator::v4::types::AppYml;
use citadel_apps::composegenerator::convert_config;
#[cfg(feature = "preprocess")]
use citadel_apps::{
    composegenerator::{load_config, v4::{utils::derive_entropy, permissions::is_allowed_by_permissions}},
    utils::flatten,
};
use clap::{Parser, Subcommand};
#[cfg(feature = "preprocess")]
use std::{io::{Read, Write}, path::Path};
use std::{process::exit};
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
        #[clap(short, long)]
        app_file: String,
        /// The app's ID
        #[clap(short, long)]
        app_name: String,
        /// The output file to save the result to
        output: String,
        /// The citadel seed file
        #[clap(short, long)]
        seed_file: Option<String>,
        /// The services that are installed as a list of comma separated values
        #[clap(short, long)]
        services: Option<String>,
    },
    /// Convert an Umbrel app (by app directory path) to a Citadel app.yml file
    /// Manual fixes may be required to make the app.yml work
    #[cfg(feature = "dev-tools")]
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
}

/// Manage apps on Citadel
#[derive(Parser)]
struct Cli {
    /// The subcommand to run
    #[clap(subcommand)]
    command: SubCommand,
}

fn main() {
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
            if port_map.get(&app_name).is_none() {
                log::error!("App not found in port map!");
                exit(1);
            }
            if !port_map.get(&app_name).unwrap().is_object() {
                log::error!("App definition in port map is invalid!");
                exit(1);
            }
            let port_map = port_map.get(&app_name).unwrap().as_object().unwrap();
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
            let app_yml = std::fs::File::open(app.as_str());
            if app_yml.is_err() {
                log::error!("Error opening app definition!");
                log::error!("{}", app_yml.err().unwrap());
                exit(1);
            }
            let mut context = Context::new();
            context.insert("services", &service_list);
            context.insert("app_name", &app_name);
            let mut tmpl = String::new();
            let reading_result = app_yml.unwrap().read_to_string(&mut tmpl);
            if reading_result.is_err() {
                log::error!("Error running templating engine on app definition!");
                log::error!("{}", reading_result.err().unwrap());
                exit(1);
            }
            let tmpl_result = Tera::one_off(tmpl.as_str(), &context, false);
            if tmpl_result.is_err() {
                log::error!("Error running templating engine on app definition!");
                log::error!("{}", tmpl_result.err().unwrap());
                exit(1);
            }
            let mut writer = std::fs::File::create(output.as_str()).unwrap();
            let writing_result = writer.write(tmpl_result.unwrap().as_bytes());
            if writing_result.is_err() {
                log::error!("Error saving file: {}!", writing_result.err().unwrap());
                exit(1);
            }
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
        #[cfg(feature = "dev-tools")]
        SubCommand::UmbrelToCitadel { app, output } => {
            let app_dir = std::fs::read_dir(&app);
            if app_dir.is_err() {
                log::error!("Error opening app dir!");
                log::error!("{}", app_dir.err().unwrap());
                exit(1);
            }
            let compose_yml = std::fs::File::open(app.clone() + "/docker-compose.yml");
            if compose_yml.is_err() {
                log::error!("Error opening docker-compose.yml!");
                log::error!("{}", compose_yml.err().unwrap());
                exit(1);
            }
            let app_yml = std::fs::File::open(app + "/umbrel-app.yml");
            if app_yml.is_err() {
                log::error!("Error opening umbrel-app.yml!");
                log::error!("{}", app_yml.err().unwrap());
                exit(1);
            }
            let app_yml_parsed: Result<
                citadel_apps::composegenerator::umbrel::types::Metadata,
                serde_yaml::Error,
            > = serde_yaml::from_reader(app_yml.unwrap());
            if app_yml_parsed.is_err() {
                log::error!("Error parsing umbrel-app.yml!");
                log::error!("{}", app_yml_parsed.err().unwrap());
                exit(1);
            }
            let compose_yml_parsed: Result<
                citadel_apps::composegenerator::compose::types::ComposeSpecification,
                serde_yaml::Error,
            > = serde_yaml::from_reader(compose_yml.unwrap());
            if compose_yml_parsed.is_err() {
                log::error!("Error parsing docker-compose.yml!");
                log::error!("{}", compose_yml_parsed.err().unwrap());
                exit(1);
            }
            let result = citadel_apps::composegenerator::umbrel::convert::convert_compose(
                compose_yml_parsed.unwrap(),
                app_yml_parsed.unwrap(),
            );
            let writer = std::fs::File::create(output).unwrap();
            let serialization_result = serde_yaml::to_writer(writer, &result);
            if serialization_result.is_err() {
                log::error!("Error saving file!");
                exit(1);
            }
        }
        #[cfg(feature = "dev-tools")]
        SubCommand::Validate { app, app_name } => {
            let app_yml = std::fs::File::open(app).expect("Error opening app definition!");
            convert_config(&app_name, &app_yml, &None).expect("App is invalid");
            log::info!("App is valid!");
        }
    }
}
