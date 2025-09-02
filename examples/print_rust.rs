use image::{GenericImage, GenericImageView};
use ptouch::{step_filter_normal, Config, ContinuousType, Matrix, Media, Model, Printer};
use qrcode::QrCode;
use std::env;

#[derive(Debug, PartialEq)]
enum PrintOption {
    TestLabelNormalRes,
    TestLabelHighRes,
    TestLabelHighResMultiple,
    TestLabelHighResMultipleQrCode,
}

impl PrintOption {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "normal" | "normal-res" | "test-label-normal-res" => Some(Self::TestLabelNormalRes),
            "high" | "high-res" | "test-label-high-res" => Some(Self::TestLabelHighRes),
            "multiple" | "high-multiple" | "test-label-high-res-multiple" => {
                Some(Self::TestLabelHighResMultiple)
            }
            "qr" | "qrcode" | "test-label-high-res-multiple-qr-code" => {
                Some(Self::TestLabelHighResMultipleQrCode)
            }
            _ => None,
        }
    }

    fn all_options() -> Vec<&'static str> {
        vec!["normal", "high", "multiple", "qr"]
    }
}

fn print_usage() {
    println!("Usage: cargo run --example print_rust [OPTION]");
    println!("Options:");
    println!("  normal     Test label with normal resolution (720x300)");
    println!("  high       Test label with high resolution (720x600)");
    println!("  multiple   Multiple high resolution labels");
    println!("  qr         Multiple high resolution labels with QR codes");
    println!("\nIf no option is provided, 'high' is used as default.");
}

fn main() {
    env_logger::Builder::from_default_env()
        .format(|buf, record| {
            use std::io::Write;
            writeln!(
                buf,
                "[{}:{}] {} - {}",
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.level(),
                record.args()
            )
        })
        .init();

    let args: Vec<String> = env::args().collect();

    let option = if args.len() > 1 {
        let arg = &args[1];

        if arg == "--help" || arg == "-h" {
            print_usage();
            return;
        }

        match PrintOption::from_str(arg) {
            Some(opt) => opt,
            None => {
                eprintln!("Error: Unknown option '{}'", arg);
                eprintln!(
                    "Available options: {}",
                    PrintOption::all_options().join(", ")
                );
                print_usage();
                return;
            }
        }
    } else {
        PrintOption::TestLabelNormalRes
    };

    println!("Running with option: {:?}", option);

    let media = Media::Continuous(ContinuousType::Continuous62);

    let config: Config = Config::new(Model::QL820NWB, "000L4G359687".to_string(), media)
        // let config: Config = Config::new(Model::QL800, "000G2G844181".to_string(), media)
        .high_resolution(false)
        .cut_at_end(true)
        .two_colors(false)
        .enable_auto_cut(1)
        .compress(false);
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

            match Printer::new(config.high_resolution(true)) {
                Ok(printer) => printer.print(vec![bw].into_iter()).unwrap(),
                Err(err) => println!("ERROR {:#?}", err),
            }
        }
        PrintOption::TestLabelHighResMultiple => {
            let file = "examples/assets/label-720-600.png";
            let label: image::DynamicImage = image::open(file).unwrap().grayscale();
            let (_, length) = label.dimensions();
            let bytes = label.to_bytes();
            let bw = step_filter_normal(80, length, bytes);

            match Printer::new(config.high_resolution(true)) {
                Ok(printer) => printer
                    .print(vec![bw.clone(), bw.clone(), bw].into_iter())
                    .unwrap(),
                Err(err) => println!("ERROR {:#?}", err),
            }
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

            let bytes = buffer.to_luma8().into_raw();
            let bw = step_filter_normal(80, length, bytes);
            self.counter = self.counter - 1;
            Some(bw)
        } else {
            None
        }
    }
}
