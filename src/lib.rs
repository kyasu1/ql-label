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

pub type Matrix = Vec<Vec<u8>>;
pub const NORMAL_PRINTER_WIDTH: u32 = 720;
pub const WIDE_PRINTER_WIDTH: u32 = 1296;
