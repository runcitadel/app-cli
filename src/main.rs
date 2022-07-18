use app_cli::composegenerator::convert_config;
use app_cli::composegenerator::v4::types::AppYml;
use clap::Parser;
use std::{process::exit, io::{Read, Write}};
use tera::{Context, Tera};

/// Manage apps on Citadel
#[derive(Parser)]
struct Cli {
    /// The subcommand to run
    command: String,
    /// The app file to run this on
    app: Option<String>,
    /// The app's name
    #[clap(short, long)]
    app_name: Option<String>,
    /// The output to run this on
    output: Option<String>,
    /// The port map file
    #[clap(short, long)]
    port_map: Option<String>,
    #[clap(short, long)]
    /// Enable verbose mode
    verbose: bool,
    #[clap(short, long)]
    services: Option<String>,
}

fn main() {
    env_logger::init();
    let args: Cli = Cli::parse();
    match args.command.as_str() {
        "convert" => {
            if args.app.is_none() {
                log::error!("No app provided!");
                exit(1);
            }
            if args.output.is_none() {
                log::error!("No output provided!");
                exit(1);
            }
            if args.port_map.is_none() {
                log::error!("No port map provided!");
                exit(1);
            }
            if args.app_name.is_none() {
                log::error!("No app name provided!");
                exit(1);
            }
            let app_yml = std::fs::File::open(args.app.unwrap().as_str());
            if app_yml.is_err() {
                log::error!("Error opening app definition!");
                log::error!("{}", app_yml.err().unwrap());
                exit(1);
            }
            let ports_json = std::fs::File::open(args.port_map.unwrap().as_str());
            if ports_json.is_err() {
                log::error!("Error opening port map!");
                log::error!("{}", ports_json.err().unwrap());
                exit(1);
            }
            let port_map: Result<serde_json::Map<String, serde_json::Value>, serde_json::Error> =
                serde_json::from_reader(ports_json.unwrap());
            if port_map.is_err() {
                log::error!("Error loading port map!");
                log::error!("{}", port_map.err().unwrap());
                exit(1);
            }
            if port_map
                .as_ref()
                .unwrap()
                .get(args.app_name.as_ref().unwrap())
                .is_none()
            {
                log::error!("App not found in port map!");
                exit(1);
            }

            if !port_map
                .as_ref()
                .unwrap()
                .get(args.app_name.as_ref().unwrap())
                .unwrap()
                .is_object()
            {
                log::error!("App definition in port map is invalid!");
                exit(1);
            }
            let main_map = port_map.unwrap();
            let current_app_map = main_map
                .get(args.app_name.as_ref().unwrap())
                .unwrap()
                .as_object()
                .unwrap();

            let mut app_definition = String::new() ;
            let reading_result = app_yml.unwrap().read_to_string(&mut app_definition);
            if let Err(error) = reading_result {
                log::error!("Error during reading: {}", error);
                exit(1);
            }
            let result = convert_config(
                &args.app_name.unwrap(),
                &app_definition,
                current_app_map,
            );
            if let Err(error) = result {
                log::error!("Error during reading: {}", error);
                exit(1);
            }
            let writer = std::fs::File::create(args.output.unwrap().as_str()).unwrap();
            let serialization_result = serde_yaml::to_writer(writer, &result.unwrap());
            if serialization_result.is_err() {
                log::error!("Error saving file!");
                exit(1);
            }
        }
        "schema" => {
            let schema = schemars::schema_for!(AppYml);
            println!("{}", serde_yaml::to_string(&schema).unwrap());
        }
        "preprocess" => {
            if args.app.is_none() {
                log::error!("No app provided!");
                exit(1);
            }
            if args.output.is_none() {
                log::error!("No output provided!");
                exit(1);
            }
            if args.app_name.is_none() {
                log::error!("No app name provided!");
                exit(1);
            }
            let services = args.services.unwrap_or_default();
            let service_list: Vec<&str> = services.split(',').collect();
            let app_yml = std::fs::File::open(args.app.unwrap().as_str());
            if app_yml.is_err() {
                log::error!("Error opening app definition!");
                log::error!("{}", app_yml.err().unwrap());
                exit(1);
            }
            let mut context = Context::new();
            context.insert("services", &service_list);
            context.insert("app_name", args.app_name.as_ref().unwrap());
            let mut tmpl = String::new() ;
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
            let mut writer = std::fs::File::create(args.output.unwrap().as_str()).unwrap();
            let writing_result = writer.write(tmpl_result.unwrap().as_bytes());
            if writing_result.is_err() {
                log::error!("Error saving file: {}!", writing_result.err().unwrap());
                exit(1);
            }
            
        }
        _ => {
            log::error!("Command not supported");
            std::process::exit(1);
        }
    }
}
