use crate::Media;
use rusb;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    UsbError(#[from] rusb::Error),

    #[error("Device is offline")]
    DeviceOffline,

    #[error("Can't read device list, permission issue ?")]
    DeviceListNotReadable,

    #[error("Device is missing endpoint")]
    MissingEndpoint,

    #[error("Received invalid response from printer")]
    InvalidResponse(usize),

    #[error("Invalid configuration parameter")]
    InvalidConfig(String),

    #[error("Installed media is invalid")]
    InvalidMedia(Media),

    #[error("Status request return no response")]
    ReadStatusTimeout,

    #[error(transparent)]
    PrinterError(PrinterError),
}

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
}
