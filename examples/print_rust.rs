use image::{GenericImage, GenericImageView, ImageBuffer, Luma};
use ptouch::{
    step_filter_normal, Config, ContinuousType, DieCutType, Matrix, Media, Model, Printer,
};
use qrcode::QrCode;
use std::path::Path;

fn main() {
    env_logger::init();

    enum PrintOption {
        TestLabelNormalRes,
        TestLabelHighRes,
        TestLabelHighResMultiple,
        TestLabelHighResMultipleQrCode,
    }

    let option = PrintOption::TestLabelHighResMultipleQrCode;

    let media = Media::Continuous(ContinuousType::Continuous62);

    let config: Config = Config::new(Model::QL800, "000G0Z714634".to_string(), media)
        .high_resolution(false)
        .cut_at_end(true)
        .two_colors(false)
        .enable_auto_cut(1);
    // .disable_auto_cut();

    match option {
        PrintOption::TestLabelNormalRes => {
            let file = "examples/assets/label-720-300.png";
            let label: image::DynamicImage = image::open(file).unwrap().grayscale();
            let (_, length) = label.dimensions();
            let bytes = label.to_bytes();
            let bw = step_filter_normal(80, length, bytes);

            if let Ok(printer) = Printer::new(config) {
                printer.print(vec![bw].into_iter()).unwrap();
            }
        }
        PrintOption::TestLabelHighRes => {
            let file = "examples/assets/label-720-600.png";
            let label: image::DynamicImage = image::open(file).unwrap().grayscale();
            let (_, length) = label.dimensions();
            let bytes = label.to_bytes();
            let bw = step_filter_normal(80, length, bytes);

            if let Ok(printer) = Printer::new(config.high_resolution(true)) {
                printer.print(vec![bw].into_iter()).unwrap();
            }
        }
        PrintOption::TestLabelHighResMultiple => {
            Printer::new(config.high_resolution(true).disable_auto_cut())
                .unwrap()
                .print(Label { counter: 2 })
                .unwrap()
        }
        PrintOption::TestLabelHighResMultipleQrCode => Printer::new(config.high_resolution(true))
            .unwrap()
            .print(Label2 { counter: 2 })
            .unwrap(),
    };
}

struct Label {
    counter: u16,
}

impl Iterator for Label {
    type Item = Matrix;

    fn next(&mut self) -> Option<Self::Item> {
        if self.counter > 0 {
            let file = "examples/label-mini.png";
            let image: image::DynamicImage = image::open(file).unwrap();
            let (_, length) = image.dimensions();
            let image = image.grayscale();

            let mut buffer = image::DynamicImage::new_luma8(ptouch::NORMAL_PRINTER_WIDTH, length);
            buffer.invert();
            buffer.copy_from(&image, 0, 0).unwrap();
            buffer.invert();
            let bytes = buffer.to_bytes();
            let bw = step_filter_normal(80, length, bytes);
            self.counter = self.counter - 1;
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
        if self.counter > 0 {
            let length = 220;
            let qrcode = QrCode::new(format!("12345-{}", self.counter + 1)).unwrap();
            let qrcode: image::ImageBuffer<image::Rgba<u8>, std::vec::Vec<u8>> = qrcode
                .render::<image::Rgba<u8>>()
                .quiet_zone(false)
                .min_dimensions(100, 200)
                .build();

            let mut buffer = image::DynamicImage::new_luma8(ptouch::NORMAL_PRINTER_WIDTH, length);
            buffer.invert();
            buffer.copy_from(&qrcode, 0, 0).unwrap();

            let bytes = buffer.to_luma().into_raw();
            let bw = step_filter_normal(80, length, bytes);
            self.counter = self.counter - 1;
            Some(bw)
        } else {
            None
        }
    }
}
