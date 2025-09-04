//! P-Touch Printer Driver
//!
//! This crate provides a printer driver for Brother P-Touch QL series label printers.
//!
//! # Example
//!
//! ```rust,no_run
//! use ptouch::{Config, ContinuousType, Media, Model, Printer};
//! 
//! let media = Media::Continuous(ContinuousType::Continuous29);
//! let model = Model::QL820NWB;
//! let config = Config::new(model, "serial".to_string(), media);
//! let printer = Printer::new(config).unwrap();
//! ```

mod error;
mod media;
mod model;
mod printer;
mod utils;

pub use crate::{
    error::{Error, PrinterError},
    media::{ContinuousType, DieCutType, Media},
    model::Model,
    printer::{Config, Printer, Status},
    utils::{convert_rgb_to_two_color, step_filter_normal, step_filter_wide, TwoColorMatrix},
};

/// Type alias for 1-bit bitmap data used by printers.
///
/// Each inner `Vec<u8>` represents a single row of pixels, with 8 pixels
/// packed into each byte. The outer Vec represents multiple rows.
/// 
/// For normal printers: each row should be 90 bytes (720 pixels / 8)
/// For wide printers: each row should be 162 bytes (1296 pixels / 8)
pub type Matrix = Vec<Vec<u8>>;

/// Width in pixels for normal P-Touch printers (QL-720NW, QL-800, QL-820NWB).
///
/// Normal printers use 720 pixels across the tape width, requiring
/// 90 bytes per row when packed into bitmap format (720 / 8 = 90).
pub const NORMAL_PRINTER_WIDTH: u32 = 720;

/// Width in pixels for wide P-Touch printers (QL-1100 series).
///
/// Wide printers use 1296 pixels across the tape width, requiring
/// 162 bytes per row when packed into bitmap format (1296 / 8 = 162).
pub const WIDE_PRINTER_WIDTH: u32 = 1296;
