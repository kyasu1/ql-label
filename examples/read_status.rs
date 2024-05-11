use ptouch::{Config, ContinuousType, Media, Model, Printer};
//
// cargo run --example read_status 1273 8349 000J9Z880381
//

fn main() {
    env_logger::init();

    /*
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 4 {
        println!("usage: read_device <vendor-id-in-base-10> <product-id-in-base-10>");
        return;
    }

    let vid: u16 = FromStr::from_str(args[1].as_ref()).unwrap(); // 1273
    let pid: u16 = FromStr::from_str(args[2].as_ref()).unwrap(); // 8349
    let serial: String = FromStr::from_str(args[3].as_ref()).unwrap();
    */
    let media = Media::Continuous(ContinuousType::Continuous62);

    let config: Config = Config::new(Model::QL800, "000G2G844181".to_string(), media)
        .high_resolution(true)
        .cut_at_end(true)
        .two_colors(false)
        .enable_auto_cut(1);

    match Printer::new(config) {
        Ok(printer) => match printer.check_status() {
            Ok(status) => println!("{:?}", status),
            Err(err) => println!("Error {:?}", err),
        },
        Err(err) => panic!("Invalid configuration settings: {}", err),
    }
}
