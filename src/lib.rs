pub use crate::{
    error::{Error, PrinterError},
    media::{ContinuousType, Media},
    model::Model,
    printer::{Config, Printer, Status},
};

//
// cargo run 1273 8349 000J9Z880381
//
// use rusb::{
//     Context, Device, DeviceDescriptor, DeviceHandle, Direction, Error, Result, TransferType,
//     UsbContext,
// };

mod error;
mod media;
mod model;
mod printer;

// /// This returns list of connected Brother P-Touch series printers.
// pub fn list_printers() -> Vec<Printer> {
//     unimplemented!()
// }

// // ///
// pub fn cancel_printing() -> Result<(), Error> {
//     unimplemented!()
// }

// ///
// ///
// pub fn print() -> Result<(), Error> {
//     unimplemented!()
// }
