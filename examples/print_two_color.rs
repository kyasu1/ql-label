use ql_label::{
    convert_rgb_to_two_color, Config, ContinuousType, Media, Model, Printer, TwoColorMatrix,
};
use std::env;

fn print_usage() {
    println!("Usage: cargo run --example print_two_color [OPTION]");
    println!("Options:");
    println!("  test       Simple two-color test pattern");
    println!("  image      Load RGB image and convert to two-color (requires image file path)");
    println!("\nIf no option is provided, 'test' is used as default.");
}

fn main() {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();
    
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

        arg.as_str()
    } else {
        "test"
    };

    println!("Running with option: {}", option);

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

    let media = Media::Continuous(ContinuousType::Continuous62Red);

    // QL-820NWBで2色印刷を有効にする
    let config: Config = Config::new(model, serial, media)
        .two_colors(true) // 2色印刷を有効化
        .high_resolution(false)
        .cut_at_end(true)
        .enable_auto_cut(1);

    match option {
        "test" => {
            // Simple test pattern
            let two_color_matrix = create_test_pattern();

            match Printer::new(config) {
                Ok(printer) => {
                    println!("Starting two-color print job...");
                    if let Err(e) = printer.print_two_color(vec![two_color_matrix].into_iter()) {
                        eprintln!("Print failed: {:?}", e);
                    } else {
                        println!("Two-color print completed successfully!");
                    }
                }
                Err(err) => eprintln!("Failed to initialize printer: {:?}", err),
            }
        }
        "image" => {
            if args.len() < 3 {
                eprintln!("Error: Image file path required");
                eprintln!("Usage: cargo run --example print_two_color image <path-to-rgb-image>");
                return;
            }

            let image_path = &args[2];
            match load_and_convert_image(image_path) {
                Ok(two_color_matrix) => match Printer::new(config) {
                    Ok(printer) => {
                        println!("Starting two-color image print job...");
                        if let Err(e) = printer.print_two_color(vec![two_color_matrix].into_iter())
                        {
                            eprintln!("Print failed: {:?}", e);
                        } else {
                            println!("Two-color image print completed successfully!");
                        }
                    }
                    Err(err) => eprintln!("Failed to initialize printer: {:?}", err),
                },
                Err(e) => eprintln!("Failed to load/convert image: {}", e),
            }
        }
        _ => {
            eprintln!("Error: Unknown option '{}'", option);
            print_usage();
        }
    }
}

fn create_test_pattern() -> TwoColorMatrix {
    let width = ql_label::NORMAL_PRINTER_WIDTH;
    let height = 300;
    let byte_width = (width + 7) / 8;

    let mut black_matrix = vec![vec![0u8; byte_width as usize]; height];
    let mut red_matrix = vec![vec![0u8; byte_width as usize]; height];

    // Create alternating stripes
    for y in 0..height {
        for x in 0..width {
            let byte_idx = (x / 8) as usize;
            let bit_idx = 7 - (x % 8);

            if y < height / 3 {
                // Top third: black text pattern
                if (x / 40) % 2 == 0 && (y / 10) % 2 == 0 {
                    black_matrix[y][byte_idx] |= 1 << bit_idx;
                }
            } else if y < 2 * height / 3 {
                // Middle third: red pattern
                if (x / 60) % 2 == 1 && (y / 15) % 2 == 1 {
                    red_matrix[y][byte_idx] |= 1 << bit_idx;
                }
            } else {
                // Bottom third: mixed pattern
                if ((x as usize) + y) % 80 < 20 {
                    if ((x as usize) + y) % 40 < 20 {
                        black_matrix[y][byte_idx] |= 1 << bit_idx;
                    } else {
                        red_matrix[y][byte_idx] |= 1 << bit_idx;
                    }
                }
            }
        }
    }

    TwoColorMatrix::new(black_matrix, red_matrix).expect("Failed to create TwoColorMatrix")
}

fn load_and_convert_image(path: &str) -> Result<TwoColorMatrix, String> {
    let img = image::open(path).map_err(|e| format!("Failed to open image: {}", e))?;
    let rgb_img = img.to_rgb8();
    let (width, height) = rgb_img.dimensions();

    println!("Loading image: {}x{} pixels", width, height);

    // Resize if needed to fit printer width
    let target_width = ql_label::NORMAL_PRINTER_WIDTH;
    let (final_img, final_width, final_height) = if width != target_width {
        let aspect_ratio = height as f32 / width as f32;
        let new_height = (target_width as f32 * aspect_ratio) as u32;

        println!(
            "Resizing image to {}x{} to fit printer",
            target_width, new_height
        );

        let resized = image::imageops::resize(
            &rgb_img,
            target_width,
            new_height,
            image::imageops::FilterType::Lanczos3,
        );
        (resized, target_width, new_height)
    } else {
        (rgb_img, width, height)
    };

    let rgb_data = final_img.as_raw();

    convert_rgb_to_two_color(final_width, final_height, rgb_data)
}
