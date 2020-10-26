use log::{debug, error, info, warn};
use rusb::{Context, Device, DeviceDescriptor, DeviceHandle, Direction, TransferType, UsbContext};
use std::time::Duration;

use crate::{
    error::{Error, PrinterError},
    media::Media,
    model::Model,
};

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
    config: Config,
}

impl Printer {
    pub fn new(config: Config) -> Result<Self, Error> {
        rusb::set_log_level(rusb::LogLevel::Debug);
        match Context::new() {
            Ok(mut context) => {
                match Self::open_device(
                    &mut context,
                    0x04F9,
                    config.model.pid(),
                    config.serial.clone(),
                ) {
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

                        // QL-800では`has_kernel_driver`が`true`となる
                        // QL-820NWBでは`has_kernel_driver`が`false`となる
                        // `has_kernel_driver`が`true`の場合に、カーネルドライバーをデタッチしないとエラーとなる
                        //
                        handle.set_auto_detach_kernel_driver(true)?;
                        let has_kernel_driver = match handle.kernel_driver_active(0) {
                            Ok(true) => {
                                handle.detach_kernel_driver(0).ok();
                                true
                            }
                            _ => false,
                        };
                        info!(" Kernel driver support is {}", has_kernel_driver);
                        handle.set_active_configuration(1)?;
                        handle.claim_interface(0)?;
                        handle.set_alternate_setting(0, 0)?;

                        Ok(Printer {
                            handle: Box::new(handle),
                            endpoint_out,
                            endpoint_in,
                            config,
                        })
                    }
                    None => Err(Error::DeviceOffline),
                }
            }
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
        match result {
            Ok(n) => {
                if n == buf.len() {
                    Ok(n)
                } else {
                    println!(
                        "write error: bytes wrote {} != bytes supplied {}",
                        n,
                        buf.len()
                    );
                    Err(Error::InvalidResponse(n))
                }
            }
            Err(e) => Err(Error::UsbError(e)),
        }
    }

    pub fn read_status(&self) -> Result<Status, Error> {
        let timeout = Duration::from_secs(1);
        let mut buf: [u8; 32] = [0x00; 32];
        let mut counter = 0;

        while counter < 10 {
            match self
                .handle
                .read_bulk(self.endpoint_in.address, &mut buf, timeout)
            {
                // TODO: Check the first 4bytes match to [0x80, 0x20, 0x42, 0x34]
                // TODO: Check the error status
                Ok(32) => {
                    let status = Status::from_buf(buf);
                    debug!("Raw status code: {:X?}", buf);
                    debug!("Parsed Status struct: {:?}", status);
                    if status.phase == Phase::Receiving {
                        return Ok(status);
                    } else {
                        std::thread::sleep(std::time::Duration::from_secs(1));
                    }
                }
                Ok(_) => {
                    std::thread::sleep(std::time::Duration::from_secs(1));
                }
                Err(e) => return Err(Error::UsbError(e)),
            };
            counter = counter + 1;
        }
        Err(Error::ReadStatusTimeout)
    }

    fn initialize(&self) -> Vec<u8> {
        let mut buf: Vec<u8> = Vec::new();
        buf.append(&mut [0x00; 400].to_vec());
        buf.append(&mut [0x1B, 0x40].to_vec());
        buf
    }

    fn set_media(&self, buf: &mut std::vec::Vec<u8>) {
        buf.append(&mut [0x1B, 0x69, 0x7A].to_vec());

        self.config.media.set_media(buf, true);
    }

    fn to_bw(width: u32, length: u32, bytes: Vec<u8>) -> Vec<Vec<u8>> {
        // convert to black and white data
        // this works fine for monochrome image in original
        // TODO: Add support for a dithering algorithm to print phots
        //
        let mut bw: Vec<Vec<u8>> = Vec::new();
        for y in 0..length {
            let mut buf: Vec<u8> = Vec::new();
            for x in 0..(width / 8) {
                let index = (1 + y) * width - (1 + x) * 8;
                let mut tmp: u8 = 0x00;
                for i in 0..8 {
                    let pixel = bytes[(index + i) as usize];
                    let value: u8 = if pixel > 80 { 1 } else { 0 };
                    tmp = tmp | (value << i);
                }
                buf.push(tmp);
            }
            /*
            let x = width / 8;
            let res = width % 8;
            if res > 0 {
                let index = (width - 8 - x * 8 + y * width) as usize;
                let mut tmp: u8 = 0x00;
                for i in 0..res {
                    tmp = tmp | (bytes[index + i as usize] as u8 & 0xF0u8) >> (7 - i);
                }
                buf.push(tmp);
            }
            */
            bw.push(buf);
        }
        bw
    }

    pub fn cancel(&self) -> Result<(), Error> {
        let buf = self.initialize();
        self.write(buf)?;
        Ok(())
    }

    pub fn print(&self, images: Vec<Vec<Vec<u8>>>) -> Result<(), Error> {
        self.request_status()?;

        match self.read_status() {
            Ok(status) => {
                status.check_media(self.config.media)?;
                self.print_label(images)?;
                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    fn print_label(&self, images: Vec<Vec<Vec<u8>>>) -> Result<(), Error> {
        let mut preamble: Vec<u8> = self.initialize();
        preamble.append(&mut [0x1B, 0x69, 0x61, 0x01].to_vec()); // Set raster command mode
        preamble.append(&mut [0x1B, 0x69, 0x21, 0x00].to_vec()); // Set auto status notificatoin mode
        preamble.append(&mut [0x4D, 0x00].to_vec()); // Set to no compression mode

        // Apply config values
        match self.config.clone().build() {
            Ok(mut buf) => preamble.append(&mut buf),
            Err(err) => return Err(err),
        }

        let mut start_flag: bool = true;
        let mut iter = images.into_iter().peekable();
        loop {
            let mut buf: Vec<u8> = Vec::new();

            match iter.next() {
                Some(image) => {
                    if start_flag {
                        buf.append(&mut preamble);
                    }

                    // ESC i z 印刷情報司令
                    self.set_media(&mut buf);
                    // Set number of raster lines
                    let len = (image.len() as u32).to_le_bytes();
                    buf.append(&mut len.to_vec());
                    if start_flag {
                        buf.append(&mut [0x00, 0x00].to_vec());
                        start_flag = false;
                    } else {
                        buf.append(&mut [0x01, 0x00].to_vec());
                    }

                    // Add raster line image data
                    for mut row in image {
                        //            buf.append(&mut [0x67, 0x00, row.len() as u8].to_vec());
                        buf.append(&mut [0x67, 0x00, 90].to_vec()); // Set 0x90 works well for no compression
                        buf.append(&mut row);
                    }

                    if iter.peek().is_some() {
                        buf.push(0x0C); // FF : Print
                        self.write(buf)?;
                        self.read_status()?;
                    } else {
                        buf.push(0x1A); // Control-Z : Print then Eject
                        self.write(buf)?;
                    }
                }
                None => {
                    break;
                }
            }
        }
        Ok(())
    }

    /// Request printer status. call this function once before printing.
    ///
    pub fn request_status(&self) -> Result<usize, Error> {
        let mut buf: Vec<u8> = self.initialize();
        buf.append(&mut [0x1b, 0x69, 0x53].to_vec());
        self.write(buf)
    }
}

///
/// Status received from the printer encoded to Rust friendly type.
///
#[derive(Debug)]
pub struct Status {
    model: Model,
    error: PrinterError,
    media: Option<Media>,
    mode: u8,
    status_type: StatusType,
    phase: Phase,
    notification: Notification,
}

impl Status {
    fn from_buf(buf: [u8; 32]) -> Self {
        Status {
            model: Model::from_code(buf[4]),
            error: PrinterError::from_buf(buf),
            media: Media::from_buf(buf),
            mode: buf[15],
            status_type: StatusType::from_code(buf[18]),
            phase: Phase::from_buf(buf),
            notification: Notification::from_code(buf[22]),
        }
    }

    pub fn check_media(self, media: Media) -> Result<(), Error> {
        if let Some(m) = self.media {
            Ok(())
        } else {
            Err(Error::InvalidMedia(media))
        }
    }
}

// StatusType

#[derive(Debug, PartialEq)]
enum StatusType {
    ReplyToRequest,
    Completed,
    Error,
    Offline,
    Notification,
    PhaseChange,
    Unknown,
}

impl StatusType {
    fn from_code(code: u8) -> StatusType {
        match code {
            0x00 => Self::ReplyToRequest,
            0x01 => Self::Completed,
            0x02 => Self::Error,
            0x04 => Self::Offline,
            0x05 => Self::Notification,
            0x06 => Self::PhaseChange,
            _ => Self::Unknown,
        }
    }
}
// Phase

#[derive(Debug, PartialEq)]
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

/// Config
///
#[derive(Debug, Clone, Copy)]
enum AutoCut {
    Enabled(u8),
    Disabled,
}

#[derive(Debug, Clone)]
pub struct Config {
    model: Model,
    serial: String,
    media: Media,
    auto_cut: AutoCut,
    two_colors: bool,
    cut_at_end: bool,
    high_resolution: bool,
    feed: u16,
}

impl Config {
    pub fn new(model: Model, serial: String, media: Media) -> Config {
        Config {
            model: model,
            serial: serial,
            media: media,
            auto_cut: AutoCut::Enabled(1),
            two_colors: false,
            cut_at_end: true,
            high_resolution: false,
            feed: media.get_default_feed_dots(),
        }
    }

    pub fn enable_auto_cut(self, size: u8) -> Self {
        Config {
            auto_cut: AutoCut::Enabled(size),
            ..self
        }
    }

    pub fn disable_auto_cut(self) -> Self {
        Config {
            auto_cut: AutoCut::Disabled,
            ..self
        }
    }

    pub fn cut_at_end(self, flag: bool) -> Self {
        Config {
            cut_at_end: flag,
            ..self
        }
    }

    pub fn high_resolution(self, high: bool) -> Self {
        Config {
            high_resolution: high,
            ..self
        }
    }

    pub fn set_feed_in_dots(self, feed: u16) -> Self {
        Config { feed, ..self }
    }

    fn build(self) -> Result<Vec<u8>, Error> {
        let mut buf: Vec<u8> = Vec::new();

        // Set feeding values in dots
        {
            match self.media.check_feed_value(self.feed) {
                Ok(feed) => {
                    buf.append(&mut [0x1B, 0x69, 0x64].to_vec());
                    buf.append(&mut feed.to_vec());
                }
                Err(msg) => return Err(Error::InvalidConfig(msg)),
            }
        }
        // Set auto cut settings
        {
            let mut various_mode: u8 = 0b0000_0000;
            let mut auto_cut_num: u8 = 1;

            if let AutoCut::Enabled(n) = self.auto_cut {
                various_mode = various_mode | 0b0100_0000;
                auto_cut_num = n;
            }

            debug!("Various mode: {:X}", various_mode);
            debug!("Auto cut num: {:X}", auto_cut_num);

            buf.append(&mut [0x1B, 0x69, 0x4D, various_mode].to_vec()); // ESC i M : Set various mode
            buf.append(&mut [0x1B, 0x69, 0x41, auto_cut_num].to_vec()); // ESC i A : Set auto cut number
        }
        // Set expanded mode
        {
            let mut expanded_mode: u8 = 0b00000000;

            if self.two_colors {
                expanded_mode = expanded_mode | 0b0000_0001;
            }

            if self.cut_at_end {
                expanded_mode = expanded_mode | 0b0000_1000;
            };

            if self.high_resolution {
                expanded_mode = expanded_mode | 0b0100_0000;
            }

            debug!("Expanded mode: {:X}", expanded_mode);

            buf.append(&mut [0x1B, 0x69, 0x4B, expanded_mode].to_vec()); // ESC i K : Set expanded mode
        }
        Ok(buf)
    }
}
