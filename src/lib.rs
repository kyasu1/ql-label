pub use crate::{
    error::{Error, PrinterError},
    printer::{Printer, Status, Config},
    model::{Model},
    media::{Media},
};

//
// cargo run 1273 8349 000J9Z880381
//
// use rusb::{
//     Context, Device, DeviceDescriptor, DeviceHandle, Direction, Error, Result, TransferType,
//     UsbContext,
// };

mod error;
mod printer;
mod media;
mod model;

/// This returns list of connected Brother P-Touch series printers.
pub fn list_printers() -> Vec<Printer> {
    unimplemented!()
}

// ///
pub fn cancel_printing() -> Result<(), Error> {
    unimplemented!()
}

///
/// 
pub fn print() -> Result<(), Error> {
    unimplemented!()
}