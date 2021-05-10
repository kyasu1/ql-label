/// choose a step filter depending on the priter width.
///
use crate::Matrix;

pub fn step_filter_normal(threashold: u8, length: u32, bytes: Vec<u8>) -> Matrix {
    step_filter(threashold, crate::NORMAL_PRINTER_WIDTH, length, bytes)
}

pub fn step_filter_wide(threashold: u8, length: u32, bytes: Vec<u8>) -> Matrix {
    step_filter(threashold, crate::WIDE_PRINTER_WIDTH, length, bytes)
}

fn step_filter(threashold: u8, width: u32, length: u32, bytes: Vec<u8>) -> Matrix {
    // convert to black and white data
    // threashold = 80 seems to work fine if original data is monochrome.
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
                let value: u8 = if pixel > threashold { 0 } else { 1 };
                tmp = tmp | (value << i);
            }
            buf.push(tmp);
        }
        bw.push(buf);
    }

    bw
}
