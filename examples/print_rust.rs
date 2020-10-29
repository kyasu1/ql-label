use image::{GenericImage, GenericImageView, ImageBuffer, Luma};
use ptouch::{
    step_filter_normal, Config, ContinuousType, DieCutType, Matrix, Media, Model, Printer,
};
use qrcode::QrCode;
use std::path::Path;

fn main() {
    env_logger::init();

    // // let file = "examples/rust-logo-256x256-blk.png";
    // // let file = "examples/test-text.png";
    // // let file = "examples/print-sample.png";
    // let file = "examples/label-mini.png";
    // // let file = "examples/PAWN_TICKET_JP.bmp";
    // // let file = "examples/label62x29.png";

    // let image: image::DynamicImage = image::open(file).unwrap();
    // let (_, length) = image.dimensions();
    // let gray = image.grayscale();

    // // canvas width is fixed 720 dots (90 bytes)
    // const WIDTH: u32 = 720;

    // // let media = Media::DieCut(DieCutType::DieCut62x29);
    // let media = Media::Continuous(ContinuousType::Continuous62);

    // let mut buffer = image::DynamicImage::new_luma8(WIDTH, length);
    // buffer.invert();
    // buffer.copy_from(&gray, 0, 0).unwrap();
    // buffer.invert();
    // let bytes = buffer.to_bytes();

    // let bw = step_filter(WIDTH, length, bytes);

    let media = Media::Continuous(ContinuousType::Continuous62);

    let config: Config = Config::new(Model::QL800, "000G0Z714634".to_string(), media)
        .high_resolution(true)
        .cut_at_end(true)
        .two_colors(false)
        // .enable_auto_cut(1);
        .disable_auto_cut();

    let label: Label2 = Label2 { counter: 0 };

    match Printer::new(config) {
        Ok(printer) => {
            // printer.print(vec![bw.clone()]).unwrap();
            printer.print(label).unwrap();
        }
        Err(err) => panic!("Error at main {}", err),
    }
}

struct Label {
    counter: u16,
}

impl Iterator for Label {
    type Item = Matrix;

    fn next(&mut self) -> Option<Self::Item> {
        if self.counter < 2 {
            let file = "examples/label-mini.png";
            let image: image::DynamicImage = image::open(file).unwrap();
            let (_, length) = image.dimensions();
            let image = image.grayscale();

            let mut buffer = image::DynamicImage::new_luma8(720, length);
            buffer.invert();
            buffer.copy_from(&image, 0, 0).unwrap();
            buffer.invert();
            let bytes = buffer.to_bytes();
            let bw = step_filter_normal(80, length, bytes);
            self.counter = self.counter + 1;
            Some(bw)
        } else {
            None
        }
    }
}

struct Label2 {
    counter: u16,
}

impl Iterator for Label2 {
    type Item = Matrix;

    fn next(&mut self) -> Option<Self::Item> {
        if self.counter < 2 {
            let length = 220;
            let code = QrCode::new(b"01234567").unwrap();
            let image: image::ImageBuffer<image::Luma<u8>, std::vec::Vec<u8>> = code
                .render::<image::Luma<u8>>()
                .quiet_zone(false)
                .min_dimensions(100, 200)
                .build();
            let dimensions = image.dimensions();
            println!("dimensions {:?}", dimensions);

            let mut buffer =
                image::DynamicImage::new_luma8(ptouch::NORMAL_PRINTER_WIDTH, length).to_luma();
            // buffer.invert();
            buffer.copy_from(&image, 0, 0).unwrap();
            // buffer.invert();
            let bytes = buffer.into_raw();
            let bw = step_filter_normal(80, length, bytes);
            self.counter = self.counter + 1;
            Some(bw)
        } else {
            None
        }
    }
}
