//! Error types for P-Touch printer operations.
//!
//! This module defines all possible errors that can occur during printer
//! communication, configuration, and print operations.

use crate::Media;
use rusb;
use thiserror::Error;

/// Main error type for P-Touch printer operations.
///
/// This enum encompasses all possible errors that can occur when using
/// the printer, from USB communication issues to printer-specific errors.
#[derive(Error, Debug)]
pub enum Error {
    /// USB communication error.
    ///
    /// Wraps underlying rusb errors for device communication issues,
    /// timeouts, or permission problems.
    #[error(transparent)]
    UsbError(#[from] rusb::Error),

    /// Printer device is not connected or not responding.
    ///
    /// This error occurs when the printer cannot be found on USB or
    /// fails to respond to initialization commands.
    #[error("Device is offline")]
    DeviceOffline,

    #[error("Can't read device list, permission issue ?")]
    DeviceListNotReadable,

    #[error("Device is missing endpoint")]
    MissingEndpoint,

    #[error("Received invalid response from printer")]
    InvalidResponse(usize),

    /// Invalid configuration parameter provided.
    ///
    /// This error occurs when configuration values are out of range
    /// or incompatible with the selected printer model.
    #[error("Invalid configuration parameter")]
    InvalidConfig(String),

    #[error("No media is installed in the printer")]
    NoMediaInstalled,

    /// Media type mismatch between configuration and installed tape.
    ///
    /// The printer has different media installed than what was specified
    /// in the configuration. Check the installed tape and update config.
    #[error("Media mismatch: expected {expected:?}, found {actual:?}")]
    MediaMismatch { expected: Media, actual: Media },

    #[error("Status request return no response")]
    ReadStatusTimeout,

    /// Print job timed out waiting for completion.
    ///
    /// The printer did not complete the print job within the expected time.
    /// This may indicate a hardware issue or very long label.
    #[error("Print job timeout waiting for completion")]
    PrintTimeout,

    #[error("Unexpected printer phase: {0:?}")]
    UnexpectedPhase(crate::printer::Phase),

    /// Hardware-level printer error.
    ///
    /// Wraps printer-specific errors reported by the device itself,
    /// such as cover open, media issues, or mechanical problems.
    #[error(transparent)]
    PrinterError(PrinterError),
}

/// Hardware-specific errors reported by the printer.
///
/// These errors are parsed from the printer's status response and indicate
/// physical problems with the device that need user intervention.
#[derive(Error, Debug)]
pub enum PrinterError {
    // Following errors are read from printer status
    #[error("No media is installed")]
    NoMedia,

    #[error("End of media")]
    EndOfMedia,

    #[error("Cutter jam")]
    CutterJam,

    #[error("Printer is in use")]
    PrinterInUse,

    #[error("Printer if offline")]
    PrinterOffline,

    #[error("Installed media is not match")]
    InvalidMedia,

    #[error("Expansion buffer is full")]
    BufferFull,

    #[error("Communication error")]
    CommunicationError,

    #[error("Cover is open")]
    CoverOpen,

    #[error("Media can not be fed")]
    FeedMediaFail,

    #[error("System error")]
    SystemError,

    #[error("Unknown error")]
    UnknownError((u8, u8)),
}

impl PrinterError {
    /// Parse printer error from 32-byte status buffer.
    ///
    /// Analyzes bytes 8 and 9 of the printer status response to determine
    /// the specific error condition reported by the hardware.
    ///
    /// # Arguments
    /// * `buf` - 32-byte status response from printer
    ///
    /// # Returns
    /// Parsed printer error or `UnknownError((0, 0))` if no error
    pub fn from_buf(buf: [u8; 32]) -> Self {
        let err_1 = buf[8];
        let err_2 = buf[9];

        match err_1 {
            0b0000_0001 => Self::NoMedia,
            0b0000_0010 => Self::EndOfMedia,
            0b0000_0100 => Self::CutterJam,
            0b0001_0000 => Self::PrinterInUse,
            0b0010_0000 => Self::PrinterOffline,
            _ => match err_2 {
                0b0000_0001 => Self::InvalidMedia,
                0b0000_0010 => Self::BufferFull,
                0b0000_0100 => Self::CommunicationError,
                0b0001_0000 => Self::CoverOpen,
                0b0100_0000 => Self::FeedMediaFail,
                0b1000_0000 => Self::SystemError,
                _ => Self::UnknownError((err_1, err_2)),
            },
        }
    }

    /// Check if this represents a "no error" state.
    ///
    /// Returns `true` if the printer is reporting no error condition.
    /// Used to distinguish between actual errors and normal status.
    ///
    /// # Returns
    /// `true` if no error is present, `false` otherwise
    pub fn is_no_error(&self) -> bool {
        matches!(self, Self::UnknownError((0, 0)))
    }
}
