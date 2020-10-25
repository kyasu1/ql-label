use ptouch::{Model, Printer};
use std::str::FromStr;
//
// cargo run --example read_status 1273 8349 000J9Z880381
//

fn main() {
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
    match Printer::new(Model::QL800, "000G0Z714634".to_string()) {
        Ok(printer) => {
            printer.initialize();
            printer.request_status();
            let result = printer.read_status();
            println!("status: {:?}", result);
        }
        Err(err) => panic!("read error {}", err),
    }
}
