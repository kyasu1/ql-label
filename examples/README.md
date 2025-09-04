# Two-Color Printing Examples

This document describes how to use the two-color printing feature with QL-820NWB printers.

## Usage

### Basic Two-Color Test Pattern

```bash
RUST_LOG=debug cargo run --example print_two_color test
```

This will print a test pattern with both black and red elements to verify two-color functionality.

### Print RGB Image as Two-Color

```bash
RUST_LOG=debug cargo run --example print_two_color image path/to/your/image.png
```

This will:
1. Load an RGB image
2. Convert red pixels (R>200, G<100, B<100) to red print color
3. Convert dark pixels (brightness < 128, excluding red) to black print color
4. Print the result as a two-color label

## Configuration Requirements

For two-color printing, the `Config` must be set with:
- `two_colors(true)` - Enable two-color printing mode
- Compatible QL-820NWB printer with red/black tape installed

## API Usage

```rust
use ptouch::{Config, ContinuousType, Media, Model, Printer, TwoColorMatrix, convert_rgb_to_two_color};

// Create config with two-color enabled
let config = Config::new(Model::QL820NWB, "serial".to_string(), media)
    .two_colors(true);

// Method 1: Use TwoColorMatrix directly
let two_color_data = TwoColorMatrix::new(black_matrix, red_matrix)?;
printer.print_two_color(vec![two_color_data].into_iter())?;

// Method 2: Convert RGB image
let two_color_data = convert_rgb_to_two_color(width, height, &rgb_bytes)?;
printer.print_two_color(vec![two_color_data].into_iter())?;
```

## Technical Details

- Black raster lines use command `0x77 0x01`
- Red raster lines use command `0x77 0x02`  
- Lines are alternated: black line, red line, black line, etc.
- Each line is 90 bytes (720 pixels / 8 bits per byte)
- Total raster count = image_height * 2 (due to alternating lines)