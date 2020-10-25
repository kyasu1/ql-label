use image::GenericImage;
use image::{GenericImageView, ImageFormat};
use ptouch::{Media, Model, Printer, Config};
use std::path::Path;

fn main() {
    let file = "examples/rust-logo-256x256-blk.png";
    let mut im: image::DynamicImage = image::open(file).unwrap();

    const PINS: u32 = 720;
    let left_pins = 0; //12;
    let width = PINS - left_pins;
    let length = 300;
    let offset_x = 12;

    let gray = im.grayscale();
    let mut buffer = image::DynamicImage::new_luma8(width, length);
    buffer.invert();
    buffer.copy_from(&gray, offset_x + 24, 35).unwrap();

    buffer.save("examples/out.png").unwrap();

    let bytes = buffer.to_bytes();
    println!("bytes size is {}", bytes.len());

    assert!(bytes.len() == (width * length) as usize);

    let mut bw: Vec<Vec<u8>> = Vec::new();

    for y in 0..length {
        let mut buf: Vec<u8> = Vec::new();
        for x in 0..(width / 8) {
            // 0 1 ... width / 8 (max 90)
            let index = (x * 8 + y * width) as usize;
            let mut tmp: u8 = 0x00;
            for i in 0..8 {
                tmp = tmp | (bytes[index + i as usize] as u8 & 0xF0u8) >> (7 - i);
            }
            buf.push(tmp);
        }
        let x = width / 8;
        let res = width % 8;
        if res > 0 {
            let index = (x * 8 + y * width) as usize;
            let mut tmp: u8 = 0x00;
            for i in 0..res {
                tmp = tmp | (bytes[index + i as usize] as u8 & 0xF0u8) >> (7 - i);
            }
            buf.push(tmp);
        }
        bw.push(buf);
    }

    println!("bw size is {}", bw.len());
    for row in bw.clone() {
        for col in row {
            print!("{:x}", col);
        }
        println!("");
    }

    match Printer::new(Model::QL800, "000G0Z714634".to_string()) {
        Ok(printer) => {
            printer.initialize();
            printer.request_status().unwrap();
            let result = printer.read_status();
            println!("status before: {:?}", result);

            let config: Config = Config::new();
            match printer.print_label(bw, config) {
                Ok(_) => println!("success"),
                Err(err) => println!("ERROR {:?}", err),
            }
            // printer.request_status().unwrap();
            let result = printer.read_status();
            println!("status after: {:?}", result);
        }
        Err(err) => panic!("read error {}", err),
    }
}

// fn to_bw() -> Vec<u8> {}
