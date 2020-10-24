pub use crate::{
    error::{Error, PrinterError},
    printer::{Model, Printer, Status},
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

/// This returns list of connected Brother P-Touch series printers.
pub fn list_printers() -> Vec<Printer> {
    unimplemented!()
}

// ///
pub fn cancel_printing() -> Result<(), error::Error> {
    unimplemented!()
}
