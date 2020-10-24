use std::{str::FromStr, time::Duration};

//
// cargo run 1273 8349 000J9Z880381
//
use rusb::{
    Context, Device, DeviceDescriptor, DeviceHandle, Direction, Error, Result, TransferType,
    UsbContext,
};

#[derive(Debug, Clone, Copy)]
struct Endpoint {
    config: u8,
    iface: u8,
    setting: u8,
    address: u8,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 4 {
        println!("usage: read_device <vendor-id-in-base-10> <product-id-in-base-10>");
        return;
    }

    let vid: u16 = FromStr::from_str(args[1].as_ref()).unwrap(); // 1273
    let pid: u16 = FromStr::from_str(args[2].as_ref()).unwrap(); // 8349
    let serial: String = FromStr::from_str(args[3].as_ref()).unwrap();

    rusb::set_log_level(rusb::LogLevel::Debug);
    match Context::new() {
        Ok(mut context) => match open_device(&mut context, vid, pid, serial.clone()) {
            Some((mut device, device_desc, mut handle)) => {
                // read_device(&mut device, &device_desc, &mut handle).unwrap();
                let endpoint_in =
                    find_readable_endpoint(&mut device, &device_desc, TransferType::Bulk)
                        .expect("No readable endpoint found");

                let endpoint_out =
                    find_writable_endpoint(&mut device, &device_desc, TransferType::Bulk)
                        .expect("No writable endpoint found");

                // match reset_ptouch(&mut handle, endpoint_out) {
                //     Ok(_) => println!("reset ptouch success"),
                //     Err(e) => panic!("something wrong: {}", e),
                // }

                match write(&mut handle, endpoint_out, [0x00; 400].to_vec()) {
                    Ok(n) => println!("n is {:?}", n),
                    Err(e) => panic!("something wrong: {}", e),
                }

                match write(&mut handle, endpoint_out, [0x1b, 0x40].to_vec()) {
                    Ok(n) => println!("n is {:?}", n),
                    Err(e) => panic!("something wrong: {}", e),
                }

                match write(&mut handle, endpoint_out, [0x1b, 0x69, 0x53].to_vec()) {
                    Ok(n) => println!("n is {:?}", n),
                    Err(e) => panic!("something wrong: {}", e),
                }

                match read_status(&mut handle, endpoint_in) {
                    Ok(status) => println!("status is {:?} bytes", status),
                    Err(e) => panic!("read error {}", e),
                }
            }
            None => println!("could not find device {:04x}:{:04x}@{}", vid, pid, serial),
        },
        Err(e) => panic!("could not initialize libusb: {}", e),
    }
}

fn open_device<T: UsbContext>(
    context: &mut T,
    vid: u16,
    pid: u16,
    serial: String,
) -> Option<(Device<T>, DeviceDescriptor, DeviceHandle<T>)> {
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
                        match handle.read_serial_number_string(language, &device_desc, timeout) {
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

#[derive(Debug)]
struct Status {
    model_code: u8,
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
            model_code: buf[4],
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

/// Send request stauts code then read back the status.
///
/// This function attempts to read the printer status by sending
fn write<T: UsbContext>(
    handle: &mut DeviceHandle<T>,
    endpoint: Endpoint,
    buf: Vec<u8>,
) -> Result<usize> {
    let timeout = Duration::from_secs(1);

    let result = handle.write_bulk(endpoint.address, &buf, timeout);
    println!("write: {:?}", result);
    match result {
        Ok(n) => {
            if n == buf.len() {
                Ok(n)
            } else {
                println!("write_bulk return unexpected number {}", n);
                Err(Error::Other)
            }
        }
        Err(e) => Err(e),
    }
}

fn read_status<T: UsbContext>(handle: &mut DeviceHandle<T>, endpoint: Endpoint) -> Result<Status> {
    let timeout = Duration::from_secs(1);
    let mut buf: [u8; 32] = [0x00; 32];

    let mut counter: u8 = 10;

    while counter > 0 {
        match handle.read_bulk(endpoint.address, &mut buf, timeout) {
            Ok(32) => return Ok(Status::from_buf(buf)),
            Ok(_) => {
                counter = counter - 1;
                continue;
            }
            Err(e) => return Err(e),
        }
    }

    Err(Error::Other)
}

fn read_device<T: UsbContext>(
    device: &mut Device<T>,
    device_desc: &DeviceDescriptor,
    handle: &mut DeviceHandle<T>,
) -> Result<()> {
    handle.reset()?;

    let timeout = Duration::from_secs(1);
    let languages = handle.read_languages(timeout)?;

    println!("Active configuration: {}", handle.active_configuration()?);
    println!("Languages: {:?}", languages);

    if languages.len() > 0 {
        let language = languages[0];

        println!(
            "Manufacturer: {:?}",
            handle
                .read_manufacturer_string(language, device_desc, timeout)
                .ok()
        );
        println!(
            "Product: {:?}",
            handle
                .read_product_string(language, device_desc, timeout)
                .ok()
        );
        println!(
            "Serial Number: {:?}",
            handle
                .read_serial_number_string(language, device_desc, timeout)
                .ok()
        );
    }

    // match find_readable_endpoint(device, device_desc, TransferType::Interrupt) {
    //     Some(endpoint) => read_endpoint(handle, endpoint, TransferType::Interrupt),
    //     None => println!("No readable interrupt endpoint"),
    // }

    match find_readable_endpoint(device, device_desc, TransferType::Bulk) {
        Some(endpoint) => read_endpoint(handle, endpoint, TransferType::Bulk),
        None => println!("No readable bulk endpoint"),
    }

    Ok(())
}

fn find_readable_endpoint<T: UsbContext>(
    device: &mut Device<T>,
    device_desc: &DeviceDescriptor,
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
                    if endpoint_desc.direction() == Direction::In
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

fn read_endpoint<T: UsbContext>(
    handle: &mut DeviceHandle<T>,
    endpoint: Endpoint,
    transfer_type: TransferType,
) {
    println!("Reading from endpoint: {:?}", endpoint);

    let has_kernel_driver = match handle.kernel_driver_active(endpoint.iface) {
        Ok(true) => {
            handle.detach_kernel_driver(endpoint.iface).ok();
            true
        }
        _ => false,
    };

    println!(" - kernel driver? {}", has_kernel_driver);

    match configure_endpoint(handle, &endpoint) {
        Ok(_) => {
            let mut buf = [0; 256];
            let timeout = Duration::from_secs(1);

            match transfer_type {
                TransferType::Interrupt => {
                    match handle.read_interrupt(endpoint.address, &mut buf, timeout) {
                        Ok(len) => {
                            println!(" - read: {:?}", &buf[..len]);
                        }
                        Err(err) => println!("could not read from endpoint: {}", err),
                    }
                }
                TransferType::Bulk => match handle.read_bulk(endpoint.address, &mut buf, timeout) {
                    Ok(len) => {
                        println!(" - read: {:?}", &buf[..len]);
                    }
                    Err(err) => println!("could not read from endpoint: {}", err),
                },
                _ => (),
            }
        }
        Err(err) => println!("could not configure endpoint: {}", err),
    }

    if has_kernel_driver {
        handle.attach_kernel_driver(endpoint.iface).ok();
    }

    handle.release_interface(endpoint.iface).unwrap();
}

fn find_writable_endpoint<T: UsbContext>(
    device: &mut Device<T>,
    device_desc: &DeviceDescriptor,
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
                    if endpoint_desc.direction() == Direction::Out
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

fn configure_endpoint<T: UsbContext>(
    handle: &mut DeviceHandle<T>,
    endpoint: &Endpoint,
) -> Result<()> {
    handle.set_active_configuration(endpoint.config)?;
    handle.claim_interface(endpoint.iface)?;
    handle.set_alternate_setting(endpoint.iface, endpoint.setting)?;
    Ok(())
}
