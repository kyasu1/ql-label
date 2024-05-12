//! P-Touch Printer Driver
//!
//! This crate provides a printer driver for Brother P-Touch QL series label printers.
//!
//! ```rust
//! use ptouch;
//!
//! let media
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
    utils::{step_filter_normal, step_filter_wide},
};

pub type Matrix = Vec<Vec<u8>>;
pub const NORMAL_PRINTER_WIDTH: u32 = 720;
pub const WIDE_PRINTER_WIDTH: u32 = 1296;
