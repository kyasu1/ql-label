use std::time::Duration;

use rusb::{Context, Device, DeviceDescriptor, DeviceHandle, Direction, TransferType, UsbContext};

use crate::error::Error;

#[derive(Debug, Clone, Copy)]
struct Endpoint {
    config: u8,
    iface: u8,
    setting: u8,
    address: u8,
}

pub struct Printer {
    handle: Box<DeviceHandle<Context>>,
    endpoint_out: Endpoint,
    endpoint_in: Endpoint,
}

impl Printer {
    pub fn new(model: Model, serial: String) -> Result<Self, Error> {
        rusb::set_log_level(rusb::LogLevel::Debug);
        match Context::new() {
            Ok(mut context) => match Self::open_device(&mut context, 0x04F9, model.pid(), serial) {
                Some((mut device, device_desc, mut handle)) => {
                    handle.reset()?;

                    let endpoint_in = match Self::find_endpoint(
                        &mut device,
                        &device_desc,
                        Direction::In,
                        TransferType::Bulk,
                    ) {
                        Some(endpoint) => endpoint,
                        None => return Err(Error::MissingEndpoint),
                    };

                    let endpoint_out = match Self::find_endpoint(
                        &mut device,
                        &device_desc,
                        Direction::Out,
                        TransferType::Bulk,
                    ) {
                        Some(endpoint) => endpoint,
                        None => return Err(Error::MissingEndpoint),
                    };

                    handle.set_auto_detach_kernel_driver(true)?;
                    let has_kernel_driver = match handle.kernel_driver_active(0) {
                        Ok(true) => {
                            handle.detach_kernel_driver(0).ok();
                            true
                        }
                        _ => false,
                    };             
                    println!(" - kernel driver? {}", has_kernel_driver);
                    handle.set_active_configuration(1)?;
                    handle.claim_interface(0)?;
                    handle.set_alternate_setting(0, 0)?;

                    Ok(Printer {
                        handle: Box::new(handle),
                        endpoint_out: endpoint_out,
                        endpoint_in: endpoint_in,
                    })
                }
                None => Err(Error::DeviceOffline),
            },
            Err(err) => Err(Error::UsbError(err)),
        }
    }

    fn open_device(
        context: &mut Context,
        vid: u16,
        pid: u16,
        serial: String,
    ) -> Option<(Device<Context>, DeviceDescriptor, DeviceHandle<Context>)> {
        let devices = match context.devices() {
            Ok(d) => d,
            Err(_) => return None,
        };
        for device in devices.iter() {
            let device_desc = match device.device_descriptor() {
                Ok(d) => d,
                Err(_) => continue,
            };
            if device_desc.vendor_id() == vid && device_desc.product_id() == pid {
                match device.open() {
                    Ok(handle) => {
                        let timeout = Duration::from_secs(1);
                        let languages = match handle.read_languages(timeout) {
                            Ok(l) => l,
                            Err(_) => return None,
                        };
                        if languages.len() > 0 {
                            let language = languages[0];
                            match handle.read_serial_number_string(language, &device_desc, timeout)
                            {
                                Ok(s) => {
                                    if s == serial {
                                        return Some((device, device_desc, handle));
                                    } else {
                                        continue;
                                    }
                                }
                                Err(_) => continue,
                            }
                        } else {
                            continue;
                        }
                    }
                    Err(_) => continue,
                }
            }
        }
        None
    }

    fn find_endpoint(
        device: &mut Device<Context>,
        device_desc: &DeviceDescriptor,
        direction: Direction,
        transfer_type: TransferType,
    ) -> Option<Endpoint> {
        for n in 0..device_desc.num_configurations() {
            let config_desc = match device.config_descriptor(n) {
                Ok(c) => c,
                Err(_) => continue,
            };
            for interface in config_desc.interfaces() {
                for interface_desc in interface.descriptors() {
                    for endpoint_desc in interface_desc.endpoint_descriptors() {
                        if endpoint_desc.direction() == direction
                            && endpoint_desc.transfer_type() == transfer_type
                        {
                            return Some(Endpoint {
                                config: config_desc.number(),
                                iface: interface_desc.interface_number(),
                                setting: interface_desc.setting_number(),
                                address: endpoint_desc.address(),
                            });
                        }
                    }
                }
            }
        }
        None
    }

    fn write(&self, buf: Vec<u8>) -> Result<usize, Error> {
        let timeout = Duration::from_secs(1);
        let result = self
            .handle
            .write_bulk(self.endpoint_out.address, &buf, timeout);
        println!("write: {:?}", result);
        match result {
            Ok(n) => {
                if n == buf.len() {
                    Ok(n)
                } else {
                    Err(Error::InvalidResponse(n))
                }
            }
            Err(e) => Err(Error::UsbError(e)),
        }
    }

    pub fn read_status(&self) -> Result<Status, Error> {
        let timeout = Duration::from_secs(1);
        let mut buf: [u8; 32] = [0x00; 32];
        let mut counter: u8 = 10;
        while counter > 0 {
            match self
                .handle
                .read_bulk(self.endpoint_in.address, &mut buf, timeout)
            {
                Ok(32) => return Ok(Status::from_buf(buf)),
                Ok(_) => {
                    counter = counter - 1;
                    continue;
                }
                Err(e) => return Err(Error::UsbError(e)),
            }
        }
        Err(Error::ReadStatusTimeout)
    }
    pub fn initialize(&self) -> Result<(), Error> {
        self.write([0x00; 400].to_vec())?;
        self.write([0x1b, 0x40].to_vec())?;
        Ok(())
    }

    pub fn request_status(&self) -> Result<usize, Error> {
        self.write([0x1b, 0x69, 0x53].to_vec())
    }

    /// Specify margin amount (feed amount)
    ///
    /// ESC i d {n1} {n2}
    ///
    /// - Margin amount (dots) = n1 + n2 * 256
    ///
    /// This setting is ignored for die-cut labels.
    ///
    pub fn set_margin(&self, n1: u8, n2: u8) -> Result<usize, Error> {
        self.write([0x1b, 0x69, 0x64, n1, n2].to_vec())
    }

    /// Helper function to set the margin amount in dots.
    ///
    /// This setting is ignored for die-cut labels.
    ///
    pub fn set_margin_with_dots(&self, dots: u16) -> Result<(), Error> {
        let bytes = dots.to_be_bytes();
        self.write([0x1b, 0x69, 0x64, bytes[1], bytes[0]].to_vec())?;
        Ok(())
    }

    /// Switch dynamic command mode. Fixed to the raster mode in this module.
    /// ESC i a 1
    ///
    pub fn set_raster_mode(&self) -> Result<(), Error> {
        self.write([0x1B, 0x69, 0x61, 0x01].to_vec())?;
        Ok(())
    }

    /// Switch automatic status notificaton mode.Model
    /// ESC i ! {n1}
    ///
    /// notify == true => 0: Notify (default)
    /// notify == false => 1: Do not notify
    ///
    pub fn set_nottification_mode(&self, notify: bool) -> Result<(), Error> {
        if notify {
            self.write([0x1B, 0x69, 0x21, 0x00].to_vec())?;
        } else {
            self.write([0x1B, 0x69, 0x21, 0x01].to_vec())?;
        }
        Ok(())
    }

    ///
    ///
    /// TODO: support compression mode
    pub fn transfer_raster(&self, buf: Vec<u8>) -> Result<(), Error> {
        self.write([0x67, 0x00, 0x90].to_vec())?;
        self.write(buf)?;
        Ok(())
    }

    ///
    ///
    /// Compression mode only ?
    pub fn transfer_raster_color(&self, buf: Vec<u8>) -> Result<(), Error> {
        self.write([0x77, 0x01, buf.len() as u8].to_vec())?;
        Ok(())
    }

    /// Zero raster graphics
    ///
    /// QL-800 does not support this command
    ///
    pub fn zero_raster(&self) -> Result<(), Error> {
        self.write([0x5A].to_vec())?;
        Ok(())
    }

    /// Print command
    ///
    /// Used as a print command at the end of pages other than the last page when multiple pages are printed.
    pub fn print(&self) -> Result<(), Error> {
        self.write([0xFF].to_vec())?;
        Ok(())
    }

    /// Print command with feeding
    ///
    /// Used as a print command at the end of the last page.
    pub fn print_last(&self) -> Result<(), Error> {
        self.write([0x1A].to_vec())?;
        Ok(())
    }

    pub fn print_information(&self) -> Result<(), Error> {
        Ok(())
    }
}

// Model
#[derive(Debug)]
pub enum Model {
    QL800,
    QL810W,
    QL820NWB,
}

impl Model {
    fn from_code(code: u8) -> Self {
        match code {
            0x38 => (Self::QL800),
            0x39 => (Self::QL810W),
            0x41 => (Self::QL820NWB),
            _ => panic!("Unknown model code {}", code),
        }
    }

    fn pid(&self) -> u16 {
        match self {
            Self::QL800 => 0x209b,
            Self::QL810W => 0x209c,
            Self::QL820NWB => 0x209d,
        }
    }
}

///
///

#[derive(Debug)]
pub struct Status {
    model: Model,
    error1: u8,
    error2: u8,
    media_width: u8,
    media_type: u8,
    mode: u8,
    media_length: u8,
    status_type: u8,
    phase: Phase,
    notification: Notification,
}

impl Status {
    fn from_buf(buf: [u8; 32]) -> Self {
        Status {
            model: Model::from_code(buf[4]),
            error1: buf[8],
            error2: buf[9],
            media_width: buf[10],
            media_type: buf[11],
            mode: buf[15],
            media_length: buf[17],
            status_type: buf[18],
            phase: Phase::from_buf(buf),
            notification: Notification::from_code(buf[22]),
        }
    }
}

// Phase

#[derive(Debug)]
enum Phase {
    Receiving,
    Printing,
    Waiting(u16),
    // Printing(u16),
}

impl Phase {
    fn from_buf(buf: [u8; 32]) -> Self {
        match buf[19] {
            0x00 => Self::Receiving,
            0x01 => Self::Printing,
            _ => Self::Waiting(0),
        }
    }
}

// Notification

#[derive(Debug)]
enum Notification {
    NotAvailable,
    CoolingStarted,
    CoolingFinished,
}

impl Notification {
    fn from_code(code: u8) -> Self {
        match code {
            0x03 => Self::CoolingStarted,
            0x04 => Self::CoolingFinished,
            _ => Self::NotAvailable,
        }
    }
}
