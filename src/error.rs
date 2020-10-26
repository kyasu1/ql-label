use crate::Media;
use rusb;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    UsbError(#[from] rusb::Error),

    #[error("Device is offline")]
    DeviceOffline,

    #[error("Device is missing endpoint")]
    MissingEndpoint,

    #[error("Received invalid response from printer")]
    InvalidResponse(usize),

    #[error("Invalid confifuration parameter")]
    InvalidConfig(String),

    #[error("Invalid media is installed")]
    InvalidMedia(Media),

    #[error("Status request return no response")]
    ReadStatusTimeout,

    #[error(transparent)]
    PrinterError(PrinterError),
}

#[derive(Error, Debug)]
pub enum PrinterError {
    // Following erros are read from printer status
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

    #[error("System error")]
    SystemError,

    #[error("Unknown error")]
    UnknownError(u16),
}

impl PrinterError {
    pub fn from_buf(buf: [u8; 32]) -> Self {
        match buf[8] {
            0x01 => Self::NoMedia,
            0x02 => Self::EndOfMedia,
            _ => match buf[9] {
                0x01 => Self::InvalidMedia,
                _ => Self::UnknownError(buf[8] as u16 + (buf[9] as u16) * 256u16),
            },
        }
    }
}
