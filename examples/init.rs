use ptouch::{Config, DieCutType, Media, Model, Printer};
//
// cargo run --example init
//

fn main() {
    env_logger::init();

    let media = Media::DieCut(DieCutType::DieCut29x90);

    let config: Config = Config::new(Model::QL820NWB, "000L4G359687".to_string(), media)
        .high_resolution(true)
        .cut_at_end(true)
        .two_colors(false)
        .enable_auto_cut(1);

    match Printer::new(config) {
        Ok(printer) => match printer.cancel() {
            Ok(()) => {
                println!("init success");
            }
            Err(err) => {
                println!("init failed {:?}", err);
            }
        },
        Err(err) => panic!("Invalid configuration settings: {}", err),
    }
}
