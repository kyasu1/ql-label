use ptouch::{Config, ContinuousType, Media, Model, Printer};
use std::env;

fn main() {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();
    
    env_logger::init();

    // Get printer configuration from environment variables
    let model_str = env::var("DEFAULT_MODEL").unwrap_or_else(|_| "QL820NWB".to_string());
    let model = match model_str.as_str() {
        "QL820NWB" => Model::QL820NWB,
        "QL800" => Model::QL800,
        "QL720NW" => Model::QL720NW,
        _ => {
            eprintln!("Unsupported model: {}. Using QL820NWB as default.", model_str);
            Model::QL820NWB
        }
    };

    let serial = env::var("SERIAL")
        .expect("SERIAL not set in environment. Please copy .env.example to .env and set your printer serial.");

    let media = Media::Continuous(ContinuousType::Continuous62);

    let config: Config = Config::new(model, serial, media)
        .high_resolution(true)
        .cut_at_end(true)
        .two_colors(false)
        .enable_auto_cut(1);

    match Printer::new(config) {
        Ok(printer) => match printer.check_status() {
            Ok(status) => println!("{:#?}", status),
            Err(err) => println!("Error {:#?}", err),
        },
        Err(err) => panic!("Invalid configuration settings: {}", err),
    }
}
