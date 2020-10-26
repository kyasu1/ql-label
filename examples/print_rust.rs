use image::GenericImage;
use image::GenericImageView;
use ptouch::{Config, ContinuousType, Media, Model, Printer};
use std::path::Path;

fn main() {
    // let file = "examples/rust-logo-256x256-blk.png";
    // let file = "examples/test-text.png";
    // let file = "examples/print-sample.png";
    let file = "examples/label-mini.png";

    let im: image::DynamicImage = image::open(file).unwrap();
    let (_, length) = im.dimensions();
    let gray = im.grayscale();

    // canvas width is fixed 720 dots (90 bytes)
    const WIDTH: u32 = 720;
    let mut buffer = image::DynamicImage::new_luma8(WIDTH, length);
    buffer.invert();
    buffer.copy_from(&gray, 0, 0).unwrap();
    buffer.invert();
    let bytes = buffer.to_bytes();

    let bw = to_bw(WIDTH, length, bytes);

    if true {
        let media = Media::Continuous(ContinuousType::Continuous62);
        let config: Config = Config::new(Model::QL800, "000G0Z714634".to_string(), media)
            .high_resolution(true)
            .set_cut_at_end(true)
            .enable_auto_cut(1);

        match Printer::new(config) {
            Ok(printer) => {
                printer.request_status().unwrap();
                match printer.read_status() {
                    Ok(result) => {
                        println!("Printer Status before: {:?}", result);
                        if result.check_media(media) {
                            match printer.print_label(vec![bw.clone()]) {
                                Ok(_) => println!("success"),
                                Err(err) => println!("ERROR {:?}", err),
                            }
                            let result = printer.read_status();
                            println!("status after: {:?}", result);
                        } else {
                            panic!("Media not much {:?}", media);
                        }
                    }
                    Err(_) => panic!("Printer not responding for the status request"),
                }
            }
            Err(err) => panic!("read error {}", err),
        }
    }
}

//
fn to_bw(width: u32, length: u32, bytes: Vec<u8>) -> Vec<Vec<u8>> {
    // convert to black and white data
    // this works fine for monochrome image in original
    // TODO: Add support for a dithering algorithm to print phots
    //
    let mut bw: Vec<Vec<u8>> = Vec::new();

    for y in 0..length {
        let mut buf: Vec<u8> = Vec::new();
        for x in 0..(width / 8) {
            let index = (1 + y) * width - (1 + x) * 8;
            let mut tmp: u8 = 0x00;
            for i in 0..8 {
                let pixel = bytes[(index + i) as usize];
                let value: u8 = if pixel > 80 { 1 } else { 0 };
                tmp = tmp | (value << i);
            }
            buf.push(tmp);
        }
        /*
        let x = width / 8;
        let res = width % 8;
        if res > 0 {
            let index = (width - 8 - x * 8 + y * width) as usize;
            let mut tmp: u8 = 0x00;
            for i in 0..res {
                tmp = tmp | (bytes[index + i as usize] as u8 & 0xF0u8) >> (7 - i);
            }
            buf.push(tmp);
        }
        */
        bw.push(buf);
    }

    bw
}
