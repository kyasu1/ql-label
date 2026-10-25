//! Utility functions for image processing and two-color printing.
//!
//! This module provides functions to convert various image formats to the
//! bitmap format required by Brother P-Touch printers.

use crate::Matrix;

/// Container for two-color (black and red) bitmap data.
///
/// This structure holds separate bitmap matrices for black and red colors,
/// used for two-color printing on compatible printers like QL-820NWB.
///
/// Both matrices must have identical dimensions and represent 1-bit bitmap data
/// packed into bytes (8 pixels per byte).
#[derive(Debug, Clone)]
pub struct TwoColorMatrix {
    pub black: Matrix,
    pub red: Matrix,
}

impl TwoColorMatrix {
    /// Create a new TwoColorMatrix from black and red bitmap data.
    ///
    /// # Arguments
    /// * `black` - Matrix containing black pixel data
    /// * `red` - Matrix containing red pixel data
    ///
    /// # Returns
    /// * `Ok(TwoColorMatrix)` - Successfully created two-color matrix
    /// * `Err(String)` - Error message if dimensions don't match
    ///
    /// # Example
    /// ```rust
    /// # use ql_label::{TwoColorMatrix, Matrix};
    /// let black_data: Matrix = vec![vec![0xFF; 90]; 300]; // 300 lines, 90 bytes each
    /// let red_data: Matrix = vec![vec![0x00; 90]; 300];   // Same dimensions
    /// 
    /// let two_color = TwoColorMatrix::new(black_data, red_data)?;
    /// # Ok::<(), String>(())
    /// ```
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
    
    /// Convert two-color data to alternating matrix format for printing.
    ///
    /// This method interleaves black and red rows to create a single matrix
    /// where black and red lines alternate. This format is required by the
    /// printer's two-color raster commands.
    ///
    /// # Returns
    /// Matrix with double the height, alternating between black and red rows
    ///
    /// # Example
    /// ```rust
    /// # use ql_label::{TwoColorMatrix, Matrix};
    /// # let black_data: Matrix = vec![vec![0xFF; 90]; 2];
    /// # let red_data: Matrix = vec![vec![0x00; 90]; 2];
    /// let two_color = TwoColorMatrix::new(black_data, red_data)?;
    /// let alternating = two_color.to_alternating_matrix();
    /// assert_eq!(alternating.len(), 4); // 2 * 2 original rows
    /// # Ok::<(), String>(())
    /// ```
    pub fn to_alternating_matrix(&self) -> Matrix {
        let mut result = Matrix::new();
        
        for (black_row, red_row) in self.black.iter().zip(self.red.iter()) {
            result.push(black_row.clone());
            result.push(red_row.clone());
        }
        
        result
    }
}

/// Convert grayscale image to 1-bit bitmap for normal-width printers (720 pixels).
///
/// This function processes grayscale image data and converts it to the 1-bit
/// bitmap format required by Brother P-Touch printers. Pixels are packed
/// 8 per byte with proper bit ordering for the printer.
///
/// # Arguments
/// * `threshold` - Grayscale threshold (0-255). Pixels below this become black (1)
/// * `length` - Image height in pixels
/// * `bytes` - Grayscale image data (width × height bytes)
///
/// # Returns
/// Matrix containing 1-bit bitmap data (`Vec<Vec<u8>>`)
///
/// # Example
/// ```rust
/// # use ql_label::{step_filter_normal, Matrix};
/// let width = 720;
/// let height = 100;
/// let grayscale_data = vec![128u8; (width * height) as usize]; // Gray image
/// 
/// let bitmap = step_filter_normal(80, height, grayscale_data);
/// assert_eq!(bitmap.len(), height as usize);
/// assert_eq!(bitmap[0].len(), 90); // 720 pixels / 8 = 90 bytes
/// ```
pub fn step_filter_normal(threshold: u8, length: u32, bytes: Vec<u8>) -> Matrix {
    step_filter(threshold, crate::NORMAL_PRINTER_WIDTH, length, bytes)
}

/// Convert grayscale image to 1-bit bitmap for wide printers (1296 pixels).
///
/// Similar to `step_filter_normal` but designed for wide printers like QL-1100 series.
/// Processes images with 1296 pixel width instead of 720.
///
/// # Arguments
/// * `threshold` - Grayscale threshold (0-255). Pixels below this become black (1)
/// * `length` - Image height in pixels
/// * `bytes` - Grayscale image data (width × height bytes)
///
/// # Returns
/// Matrix containing 1-bit bitmap data (`Vec<Vec<u8>>`)
///
/// # Example
/// ```rust
/// # use ql_label::{step_filter_wide, Matrix, WIDE_PRINTER_WIDTH};
/// let width = WIDE_PRINTER_WIDTH;
/// let height = 100;
/// let grayscale_data = vec![128u8; (width * height) as usize];
/// 
/// let bitmap = step_filter_wide(80, height, grayscale_data);
/// assert_eq!(bitmap.len(), height as usize);
/// assert_eq!(bitmap[0].len(), 162); // 1296 pixels / 8 = 162 bytes
/// ```
pub fn step_filter_wide(threshold: u8, length: u32, bytes: Vec<u8>) -> Matrix {
    step_filter(threshold, crate::WIDE_PRINTER_WIDTH, length, bytes)
}

/// Convert grayscale image to 1-bit bitmap for normal-width printers (720 pixels)
/// using Floyd–Steinberg dithering.
///
/// Printers can only print black or white dots, so shades of gray are simulated
/// by varying the density of black dots. Use this instead of `step_filter_normal`
/// for photos, gradients or logos with colored areas that would otherwise be
/// lost by a plain threshold.
///
/// # Arguments
/// * `length` - Image height in pixels
/// * `bytes` - Grayscale image data (width × height bytes)
///
/// # Returns
/// Matrix containing 1-bit bitmap data (`Vec<Vec<u8>>`)
///
/// # Example
/// ```rust
/// # use ql_label::{dither_filter_normal, Matrix};
/// let width = 720;
/// let height = 100;
/// let grayscale_data = vec![128u8; (width * height) as usize]; // Gray image
///
/// let bitmap = dither_filter_normal(height, grayscale_data);
/// assert_eq!(bitmap.len(), height as usize);
/// assert_eq!(bitmap[0].len(), 90); // 720 pixels / 8 = 90 bytes
/// ```
pub fn dither_filter_normal(length: u32, bytes: Vec<u8>) -> Matrix {
    dither_filter(crate::NORMAL_PRINTER_WIDTH, length, bytes)
}

/// Convert grayscale image to 1-bit bitmap for wide printers (1296 pixels)
/// using Floyd–Steinberg dithering.
///
/// Similar to `dither_filter_normal` but designed for wide printers like QL-1100 series.
///
/// # Arguments
/// * `length` - Image height in pixels
/// * `bytes` - Grayscale image data (width × height bytes)
///
/// # Returns
/// Matrix containing 1-bit bitmap data (`Vec<Vec<u8>>`)
///
/// # Example
/// ```rust
/// # use ql_label::{dither_filter_wide, Matrix, WIDE_PRINTER_WIDTH};
/// let width = WIDE_PRINTER_WIDTH;
/// let height = 100;
/// let grayscale_data = vec![128u8; (width * height) as usize];
///
/// let bitmap = dither_filter_wide(height, grayscale_data);
/// assert_eq!(bitmap.len(), height as usize);
/// assert_eq!(bitmap[0].len(), 162); // 1296 pixels / 8 = 162 bytes
/// ```
pub fn dither_filter_wide(length: u32, bytes: Vec<u8>) -> Matrix {
    dither_filter(crate::WIDE_PRINTER_WIDTH, length, bytes)
}

fn dither_filter(width: u32, length: u32, bytes: Vec<u8>) -> Matrix {
    step_filter(128, width, length, floyd_steinberg(width, length, &bytes))
}

/// Floyd–Steinberg error diffusion: turns grayscale pixels into pure black (0)
/// or white (255) pixels, spreading the quantization error to the neighbours.
fn floyd_steinberg(width: u32, length: u32, bytes: &[u8]) -> Vec<u8> {
    let (w, h) = (width as usize, length as usize);
    let mut buf: Vec<f32> = bytes[..w * h].iter().map(|&b| b as f32).collect();

    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            let old = buf[i];
            let new = if old < 128.0 { 0.0 } else { 255.0 };
            let err = old - new;
            buf[i] = new;

            if x + 1 < w {
                buf[i + 1] += err * 7.0 / 16.0;
            }
            if y + 1 < h {
                if x > 0 {
                    buf[i + w - 1] += err * 3.0 / 16.0;
                }
                buf[i + w] += err * 5.0 / 16.0;
                if x + 1 < w {
                    buf[i + w + 1] += err * 1.0 / 16.0;
                }
            }
        }
    }

    buf.into_iter().map(|v| v as u8).collect()
}

fn step_filter(threshold: u8, width: u32, length: u32, bytes: Vec<u8>) -> Matrix {
    // convert to black and white data
    // threshold = 80 seems to work fine if original data is monochrome.
    // For photos use `dither_filter_normal` / `dither_filter_wide` instead.
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

/// Convert RGB image data to two-color bitmap for printing.
///
/// This function analyzes RGB pixel data and separates it into black and red
/// components suitable for two-color printing. Uses color detection algorithms
/// to classify pixels as red, black, or white (not printed).
///
/// # Color Detection Rules
/// - **Red pixels**: R > 200, G < 100, B < 100
/// - **Black pixels**: Brightness < 128 (excluding red pixels)
/// - **White pixels**: Everything else (not printed)
///
/// # Arguments
/// * `width` - Image width in pixels
/// * `height` - Image height in pixels  
/// * `rgb_data` - RGB image data (width × height × 3 bytes)
///
/// # Returns
/// * `Ok(TwoColorMatrix)` - Successfully converted image data
/// * `Err(String)` - Error if data size doesn't match dimensions
///
/// # Example
/// ```rust
/// # use ql_label::{convert_rgb_to_two_color};
/// let width = 720;
/// let height = 100;
/// // Create simple RGB data: red stripe at top, black at bottom
/// let mut rgb_data = vec![];
/// for y in 0..height {
///     for x in 0..width {
///         if y < height / 2 {
///             rgb_data.extend_from_slice(&[255, 0, 0]); // Red
///         } else {
///             rgb_data.extend_from_slice(&[0, 0, 0]);   // Black
///         }
///     }
/// }
/// 
/// let two_color = convert_rgb_to_two_color(width, height, &rgb_data)?;
/// # Ok::<(), String>(())
/// ```
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

#[cfg(test)]
mod tests {
    use super::*;

    fn black_ratio(matrix: &Matrix) -> f32 {
        let total: u32 = matrix.iter().map(|row| row.len() as u32 * 8).sum();
        let black: u32 = matrix
            .iter()
            .flatten()
            .map(|byte| byte.count_ones())
            .sum();
        black as f32 / total as f32
    }

    #[test]
    fn test_dither_solid_colors() {
        let width = crate::NORMAL_PRINTER_WIDTH;
        let white = dither_filter_normal(10, vec![255u8; (width * 10) as usize]);
        assert_eq!(black_ratio(&white), 0.0);

        let black = dither_filter_normal(10, vec![0u8; (width * 10) as usize]);
        assert_eq!(black_ratio(&black), 1.0);
    }

    #[test]
    fn test_dither_preserves_gray_level() {
        let width = crate::NORMAL_PRINTER_WIDTH;
        let height = 100;

        // 25%, 50% and 75% gray should give roughly the same ratio of black dots
        for (gray, expected) in [(191u8, 0.25f32), (128, 0.5), (64, 0.75)] {
            let bitmap = dither_filter_normal(height, vec![gray; (width * height) as usize]);
            let ratio = black_ratio(&bitmap);
            assert!(
                (ratio - expected).abs() < 0.02,
                "gray {} gave black ratio {}, expected ~{}",
                gray,
                ratio,
                expected
            );
        }
    }

    #[test]
    fn test_dither_differs_from_threshold() {
        // A flat gray thresholded at 128 is all white or all black,
        // while dithering must produce a mix of both
        let width = crate::NORMAL_PRINTER_WIDTH;
        let bitmap = dither_filter_normal(10, vec![160u8; (width * 10) as usize]);
        let ratio = black_ratio(&bitmap);
        assert!(ratio > 0.0 && ratio < 1.0);
    }
}
