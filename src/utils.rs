/// choose a step filter depending on the priter width.
///
use crate::Matrix;

#[derive(Debug, Clone)]
pub struct TwoColorMatrix {
    pub black: Matrix,
    pub red: Matrix,
}

impl TwoColorMatrix {
    pub fn new(black: Matrix, red: Matrix) -> Result<Self, String> {
        if black.len() != red.len() {
            return Err("Black and red matrices must have the same height".to_string());
        }
        
        for (i, (black_row, red_row)) in black.iter().zip(red.iter()).enumerate() {
            if black_row.len() != red_row.len() {
                return Err(format!("Row {} has mismatched widths", i));
            }
        }
        
        Ok(TwoColorMatrix { black, red })
    }
    
    pub fn to_alternating_matrix(&self) -> Matrix {
        let mut result = Matrix::new();
        
        for (black_row, red_row) in self.black.iter().zip(self.red.iter()) {
            result.push(black_row.clone());
            result.push(red_row.clone());
        }
        
        result
    }
}

pub fn step_filter_normal(threshold: u8, length: u32, bytes: Vec<u8>) -> Matrix {
    step_filter(threshold, crate::NORMAL_PRINTER_WIDTH, length, bytes)
}

pub fn step_filter_wide(threshold: u8, length: u32, bytes: Vec<u8>) -> Matrix {
    step_filter(threshold, crate::WIDE_PRINTER_WIDTH, length, bytes)
}

fn step_filter(threshold: u8, width: u32, length: u32, bytes: Vec<u8>) -> Matrix {
    // convert to black and white data
    // threshold = 80 seems to work fine if original data is monochrome.
    // TODO: Add support for a dithering algorithm to print photos
    //
    // width must be
    let mut bw: Vec<Vec<u8>> = Vec::new();

    for y in 0..length {
        let mut buf: Vec<u8> = Vec::new();
        for x in 0..(width / 8) {
            let index = (1 + y) * width - (1 + x) * 8;
            let mut tmp: u8 = 0x00;
            for i in 0..8 {
                let pixel = bytes[(index + i) as usize];
                let value: u8 = if pixel > threshold { 0 } else { 1 };
                tmp = tmp | (value << i);
            }
            buf.push(tmp);
        }
        bw.push(buf);
    }

    bw
}

pub fn convert_rgb_to_two_color(
    width: u32,
    height: u32,
    rgb_data: &[u8],
) -> Result<TwoColorMatrix, String> {
    if rgb_data.len() != (width * height * 3) as usize {
        return Err("RGB data size doesn't match width * height * 3".to_string());
    }

    let mut black_matrix = Matrix::new();
    let mut red_matrix = Matrix::new();

    for y in 0..height {
        let mut black_row = vec![0u8; (width + 7) as usize / 8];
        let mut red_row = vec![0u8; (width + 7) as usize / 8];

        for x in 0..(width / 8) {
            // Use same indexing as step_filter to match existing behavior
            let base_index = (1 + y) * width - (1 + x) * 8;
            let mut black_byte: u8 = 0x00;
            let mut red_byte: u8 = 0x00;
            
            for i in 0..8 {
                let pixel_index = ((base_index + i) * 3) as usize;
                if pixel_index + 2 < rgb_data.len() {
                    let r = rgb_data[pixel_index];
                    let g = rgb_data[pixel_index + 1];
                    let b = rgb_data[pixel_index + 2];

                    if is_red_pixel(r, g, b) {
                        red_byte |= 1 << i;
                    } else if is_black_pixel(r, g, b) {
                        black_byte |= 1 << i;
                    }
                }
            }
            
            black_row[x as usize] = black_byte;
            red_row[x as usize] = red_byte;
        }

        black_matrix.push(black_row);
        red_matrix.push(red_row);
    }

    TwoColorMatrix::new(black_matrix, red_matrix)
}

fn is_red_pixel(r: u8, g: u8, b: u8) -> bool {
    r > 200 && g < 100 && b < 100
}

fn is_black_pixel(r: u8, g: u8, b: u8) -> bool {
    let brightness = ((r as u32 + g as u32 + b as u32) / 3) as u8;
    brightness < 128 && !is_red_pixel(r, g, b)
}
