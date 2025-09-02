use log::{debug, info};
use rusb::{Context, Device, DeviceDescriptor, DeviceHandle, Direction, TransferType, UsbContext};
use std::time::Duration;

use crate::{
    error::{Error, PrinterError},
    media::Media,
    model::Model,
    Matrix,
};

// Vendoer id of Brother Industries, Ltd
const VENDOR_ID: u16 = 0x04f9;

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
        // rusb::set_log_level(rusb::LogLevel::Debug);
        match Context::new() {
            Ok(mut context) => {
                match Self::open_device(&mut context, config.model.pid(), config.serial.clone()) {
                    Ok((mut device, device_desc, handle)) => {
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
                    Err(err) => {
                        debug!("[{}:{}] {:?}", file!(), line!(), err);
                        Err(Error::DeviceOffline)
                    }
                }
            }
            Err(err) => Err(Error::UsbError(err)),
        }
    }

    fn open_device(
        context: &mut Context,
        pid: u16,
        serial: String,
    ) -> Result<(Device<Context>, DeviceDescriptor, DeviceHandle<Context>), Error> {
        let devices = context.devices()?;

        if devices.is_empty() {
            debug!("Failed to read device list");
            return Err(Error::DeviceListNotReadable);
        }
        for device in devices.iter() {
            let device_desc = match device.device_descriptor() {
                Ok(d) => d,
                Err(err) => {
                    debug!("{:#?}", err);
                    continue;
                }
            };
            debug!(
                "vender_id: {:x},  product_id: {:x}",
                device_desc.vendor_id(),
                device_desc.product_id()
            );
            if device_desc.vendor_id() == VENDOR_ID && device_desc.product_id() == pid {
                match device.open() {
                    Ok(handle) => {
                        let timeout = Duration::from_secs(1);
                        let languages = handle.read_languages(timeout)?;

                        if languages.len() > 0 {
                            let language = languages[0];
                            match handle.read_serial_number_string(language, &device_desc, timeout)
                            {
                                Ok(s) => {
                                    if s == serial {
                                        debug!("Found a printer with the serial number {serial}");
                                        return Ok((device, device_desc, handle));
                                    } else {
                                        continue;
                                    }
                                }
                                Err(err) => {
                                    debug!("Failed to read serial number string: {:?}", err);
                                    continue;
                                }
                            }
                        } else {
                            continue;
                        }
                    }
                    Err(err) => {
                        debug!("Failed to open device: {:?}", err);
                        continue;
                    }
                }
            }
        }
        debug!("No device match with this serial: {:?}", serial);
        Err(Error::DeviceOffline)
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

    fn write(&self, buf: Vec<u8>) -> Result<(), Error> {
        let timeout = Duration::from_secs(3);
        let result = self
            .handle
            .write_bulk(self.endpoint_out.address, &buf, timeout);
        match result {
            Ok(n) => {
                if n == buf.len() {
                    debug!(
                        "wrote {n} bytes to the endpoint {}",
                        self.endpoint_out.address
                    );
                    Ok(())
                } else {
                    debug!(
                        "write error: bytes wrote {} != bytes supplied {}, possibly timeout ?",
                        n,
                        buf.len()
                    );
                    Err(Error::InvalidResponse(n))
                }
            }
            Err(e) => Err(Error::UsbError(e)),
        }
    }

    /// Read printer status.
    ///
    /// This method is convenient for inspection when a new media is added.
    ///
    pub fn check_status(&self) -> Result<Status, Error> {
        self.request_status()?;
        self.read_status()
    }

    fn read_status(&self) -> Result<Status, Error> {
        self.read_status_with_timeout(Duration::from_millis(1000))
    }

    fn read_status_with_timeout(&self, timeout: Duration) -> Result<Status, Error> {
        let mut buf: [u8; 32] = [0x00; 32];
        let mut counter = 0;

        debug!("reading from endpoint_in {:#?}", self.endpoint_in);
        while counter < 100000 {
            match self
                .handle
                .read_bulk(self.endpoint_in.address, &mut buf, timeout)
            {
                // TODO: Check the first 4bytes match to [0x80, 0x20, 0x42, 0x34]
                // TODO: Check the error status
                //
                // buf is pouplated with 32 bytes of data
                Ok(32) => {
                    let status = Status::from_buf(buf);
                    debug!("Raw status code: {:X?}", buf);
                    debug!("Parsed Status struct: {:?}", status);
                    return Ok(status);
                }
                Ok(x) => {
                    debug!("Waiting {counter} {x}");
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }
                Err(e) => return Err(Error::UsbError(e)),
            };
            counter = counter + 1;
        }
        Err(Error::ReadStatusTimeout)
    }

    fn wait_for_print_completion(&self) -> Result<(), Error> {
        let mut attempts = 0;
        const MAX_ATTEMPTS: u32 = 100; // 約5秒のタイムアウト
        
        debug!("Waiting for print completion...");
        
        loop {
            let status = self.read_status_with_timeout(Duration::from_millis(100))?;
            debug!("Print completion check: status_type={:?}, phase={:?}, error={:?}", 
                   status.status_type, status.phase, status.error);
            
            // エラー状態の即座検出
            if !status.error.is_no_error() {
                debug!("Print error detected: {:?}", status.error);
                return Err(Error::PrinterError(status.error));
            }
            
            match (status.status_type, status.phase) {
                // エラー状態の即座検出
                (StatusType::Error, _) => {
                    debug!("Error status type detected");
                    return Err(Error::PrinterError(status.error));
                }
                
                // 印刷完了 -> 受信待機への遷移を待つ
                (StatusType::Completed, Phase::Printing) => {
                    debug!("Print completed, checking for transition to receiving state");
                    // 完了後、受信状態への遷移を確認
                    std::thread::sleep(Duration::from_millis(100));
                    let final_status = self.read_status_with_timeout(Duration::from_millis(500))?;
                    if matches!(final_status.phase, Phase::Receiving) {
                        debug!("Successfully transitioned to receiving state");
                        return Ok(());
                    }
                    debug!("Still waiting for transition to receiving state, current phase: {:?}", final_status.phase);
                }
                
                // 既に受信状態に戻っている（即座完了）
                (StatusType::PhaseChange, Phase::Receiving) => {
                    debug!("Already transitioned to receiving state");
                    return Ok(());
                }
                
                // まだ印刷中
                (StatusType::PhaseChange, Phase::Printing) => {
                    debug!("Still printing, continuing to wait");
                    // 短い待機で継続監視
                    std::thread::sleep(Duration::from_millis(50));
                }
                
                // 予期しない状態
                _ => {
                    debug!("Unexpected status during print completion: {:#?}", status);
                    std::thread::sleep(Duration::from_millis(100));
                }
            }
            
            attempts += 1;
            if attempts >= MAX_ATTEMPTS {
                debug!("Print completion timeout after {} attempts", attempts);
                return Err(Error::PrintTimeout);
            }
        }
    }

    fn initialize(&self) -> Vec<u8> {
        let mut buf: Vec<u8> = Vec::new();
        buf.append(&mut [0x00; 400].to_vec());
        buf.append(&mut [0x1B, 0x40].to_vec());
        buf
    }

    fn set_media(&self, buf: &mut std::vec::Vec<u8>, raster_count: u32) {
        buf.extend_from_slice(&[0x1B, 0x69, 0x7A]); // ESC i z

        // n1: 有効フラグ (用紙種類+幅+長さ+ラスター数)
        let valid_flags = 0x02 | 0x04 | 0x08 | 0x40;
        buf.push(valid_flags);

        // n2: 用紙種類 (長尺:0x0A, ダイカット:0x0C)
        let media_type = match self.config.media {
            Media::Continuous(_) => 0x0A,
            Media::DieCut(_) => 0x0B,
        };
        buf.push(media_type);

        // n3, n4: 用紙幅・長さ (mm)
        let spec = self.config.media.spec();
        buf.push(spec.width_mm());
        buf.push(spec.length_mm());

        // n5-n8: ラスター数 (リトルエンディアン)
        let raster_bytes = raster_count.to_le_bytes();
        buf.extend_from_slice(&raster_bytes);

        // n9: 先頭ページフラグ (0=先頭ページ)
        buf.push(0x00);

        // n10: 固定値
        buf.push(0x00);
    }

    /// Cancel printing
    ///
    pub fn cancel(&self) -> Result<(), Error> {
        let buf = self.initialize();
        self.write(buf)?;
        Ok(())
    }

    /// Print labels
    ///
    ///
    pub fn print(&self, images: impl Iterator<Item = Matrix>) -> Result<(), Error> {
        log::debug!("request get status");

        self.request_status()?;

        match self.read_status() {
            Ok(status) => {
                log::debug!("check correct mediat installed");
                status.check_media(self.config.media)?;

                log::debug!("start printing labels");
                self.print_label(images)?;
                Ok(())
            }
            Err(err) => {
                log::debug!("Error when reading request status");
                log::debug!("print error {:?}", err);
                Err(err)
            }
        }
    }

    fn print_label(&self, images: impl Iterator<Item = Matrix>) -> Result<(), Error> {
        let mut preamble: Vec<u8> = self.initialize();
        preamble.append(&mut [0x1B, 0x69, 0x61, 0x01].to_vec()); // Set raster command mode
        preamble.append(&mut [0x1B, 0x69, 0x21, 0x00].to_vec()); // Set auto status notificatoin mode
                                                                 //
                                                                 // Apply config values
        match self.config.clone().build() {
            Ok(mut buf) => preamble.append(&mut buf),
            Err(err) => return Err(err),
        }

        if self.config.compress {
            preamble.append(&mut [0x4D, 0x02].to_vec()); // Set to pack bits compression mode
        } else {
            preamble.append(&mut [0x4D, 0x00].to_vec()); // Set to no compression mode
        }

        debug!("{:?}", self.config);

        let mut start_flag: bool = true;
        let mut color = false;

        let mut iter = images.into_iter().peekable();

        loop {
            let mut buf: Vec<u8> = Vec::new();

            match iter.next() {
                Some(image) => {
                    if start_flag {
                        buf.append(&mut preamble);
                    }

                    // ESC i z 印刷情報司令
                    let raster_count = if self.config.two_colors {
                        (image.len() / 2) as u32
                    } else {
                        image.len() as u32
                    };
                    self.set_media(&mut buf, raster_count);
                    if start_flag {
                        buf.append(&mut [0x00, 0x00].to_vec());
                        start_flag = false;
                    } else {
                        buf.append(&mut [0x01, 0x00].to_vec());
                    }

                    // Add raster line image data
                    if self.config.two_colors {
                        for mut row in image {
                            if color {
                                buf.append(&mut [0x77, 0x01, 90].to_vec());
                                buf.append(&mut row);
                                color = !color;
                            } else {
                                buf.append(&mut [0x77, 0x02, 90].to_vec());
                                buf.append(&mut row);
                                color = !color;
                            }
                        }
                    } else {
                        if self.config.compress {
                            for row in image {
                                let mut packed = Self::pack_bits(&row);
                                let len = packed.len() as u8;
                                buf.append(&mut [0x67, 0x00, len].to_vec());
                                buf.append(&mut packed);
                            }
                        } else {
                            for mut row in image {
                                buf.append(&mut [0x67, 0x00, 90].to_vec());
                                buf.append(&mut row);
                            }
                        }
                    }

                    if iter.peek().is_some() {
                        buf.push(0x0C); // FF : Print
                        self.write(buf)?;
                        let status = self.read_status()?;
                        debug!("the status after printing a page {:#?}", status);
                    } else {
                        buf.push(0x1A); // Control-Z : Print then Eject
                        self.write(buf)?;
                        debug!("Sent eject command, waiting for completion...");
                        
                        // 改善されたステータス待機
                        self.wait_for_print_completion()?;
                        debug!("Print job completed successfully");
                        
                        self.invalidate()?;
                    }
                }
                None => {
                    break;
                }
            }
        }
        Ok(())
    }

    /// TIFF PackBits圧縮アルゴリズム（Brother QL仕様準拠）
    ///
    /// 仕様:
    /// - 同一データ連続：個数-1を負数で指定 + データ1バイト
    /// - 異なるデータ連続：個数-1を正数で指定 + 全データ
    /// - 90バイト超過時は非圧縮として91バイト送信
    fn pack_bits(data: &[u8]) -> Vec<u8> {
        // 入力データが90バイト固定でない場合はそのまま返す
        if data.len() != 90 {
            return data.to_vec();
        }

        let mut packed = Vec::new();
        let mut i = 0;

        while i < data.len() {
            // Run-length encoding (RLE)のチェック
            let mut run_length = 1;
            let run_value = data[i];

            // 同じ値の連続をカウント（最大128個まで）
            while i + run_length < data.len()
                && run_length < 128
                && data[i + run_length] == run_value
            {
                run_length += 1;
            }

            // RLEが効果的な場合（2個以上の連続）
            if run_length >= 2 {
                // 負数で圧縮指示: -(count-1)
                packed.push((-(run_length as i8 - 1)) as u8);
                packed.push(run_value);
                i += run_length;
            } else {
                // リテラル実行のチェック
                let start_pos = i;
                let mut literal_length = 1;

                // リテラル実行の最適な長さを決定
                while i + literal_length < data.len() && literal_length < 128 {
                    // 次の位置で2個以上同じ値が続く場合は、ここでリテラル実行を終了
                    if i + literal_length + 1 < data.len()
                        && data[i + literal_length] == data[i + literal_length + 1]
                    {
                        break;
                    }
                    literal_length += 1;
                }

                // リテラル実行: 正数で非圧縮指示
                packed.push((literal_length - 1) as u8);
                packed.extend_from_slice(&data[start_pos..start_pos + literal_length]);
                i += literal_length;
            }
        }

        // 重要な最適化: 90バイト超過時は非圧縮として91バイト返す
        if packed.len() > 90 {
            debug!("Compression ineffective, using uncompressed data");
            let mut result = Vec::with_capacity(91);
            result.push(89); // 90-1 = 89（90バイトの非圧縮指示）
            result.extend_from_slice(data);
            result
        } else {
            debug!(
                "Compression effective: {} -> {} bytes",
                data.len(),
                packed.len()
            );
            packed
        }
    }

    fn request_status(&self) -> Result<(), Error> {
        let mut buf: Vec<u8> = self.initialize();
        buf.append(&mut [0x1b, 0x69, 0x53].to_vec());
        self.write(buf)
    }

    fn invalidate(&self) -> Result<(), Error> {
        let buf: Vec<u8> = self.initialize();
        self.write(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pack_bits_compression() {
        // テスト1: 効果的な圧縮（同一データ連続）
        let all_zeros = vec![0u8; 90];
        let compressed = Printer::pack_bits(&all_zeros);
        println!(
            "All zeros: {} -> {} bytes",
            all_zeros.len(),
            compressed.len()
        );
        assert!(compressed.len() < all_zeros.len(), "圧縮が効果的でない");

        // テスト2: 非効果的な圧縮（ランダムデータ）
        let random_data: Vec<u8> = (0..90).map(|i| (i * 37 + 17) as u8).collect();
        let compressed_random = Printer::pack_bits(&random_data);
        println!(
            "Random data: {} -> {} bytes",
            random_data.len(),
            compressed_random.len()
        );

        // テスト3: 91バイト制限の確認
        if compressed_random.len() > 90 {
            println!("91バイト制限により非圧縮データが返される");
            assert_eq!(compressed_random.len(), 91); // 89 + 90バイトの元データ
            assert_eq!(compressed_random[0], 89); // 非圧縮指示
        }

        // テスト4: 混合パターン（部分的な圧縮効果）
        let mut mixed_data = vec![0u8; 30];
        mixed_data.extend(vec![255u8; 30]);
        mixed_data.extend((0..30).map(|i| i as u8));
        let compressed_mixed = Printer::pack_bits(&mixed_data);
        println!(
            "Mixed data: {} -> {} bytes",
            mixed_data.len(),
            compressed_mixed.len()
        );
    }

    #[test]
    fn test_pack_bits_edge_cases() {
        // エッジケース1: 空のデータ
        let empty_data = vec![];
        let compressed_empty = Printer::pack_bits(&empty_data);
        assert_eq!(compressed_empty, empty_data);

        // エッジケース2: 90バイト以外のサイズ
        let wrong_size = vec![42u8; 50];
        let compressed_wrong = Printer::pack_bits(&wrong_size);
        assert_eq!(compressed_wrong, wrong_size);

        // エッジケース3: 単一バイトの繰り返し（最大圧縮）
        let single_byte = vec![42u8; 90];
        let compressed_single = Printer::pack_bits(&single_byte);
        assert_eq!(compressed_single.len(), 2); // 長さ指示 + データ
        assert_eq!(compressed_single[0], (-(90i8 - 1)) as u8); // -89
        assert_eq!(compressed_single[1], 42);
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
    id: u8,
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
            id: buf[14],
        }
    }

    pub fn check_media(self, expected_media: Media) -> Result<(), Error> {
        match self.media {
            Some(actual_media) => {
                if actual_media == expected_media {
                    Ok(())
                } else {
                    Err(Error::MediaMismatch {
                        expected: expected_media,
                        actual: actual_media,
                    })
                }
            }
            None => Err(Error::NoMediaInstalled),
        }
    }
}

// StatusType

#[derive(Debug, PartialEq, Clone, Copy)]
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

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Phase {
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
    compress: bool,
}

impl Config {
    /// Initialize configuration data with default values.
    ///
    /// This method receives model and media.  They are not modifiable after the initialization.
    ///
    /// # Example
    ///
    ///
    /// ```
    /// let media = Continuous(Continuous29);
    /// let model = Model:QL800;
    /// let config = Config::new(model, media);
    /// ```
    ///
    pub fn new(model: Model, serial: String, media: Media) -> Config {
        Config {
            model,
            serial,
            media,
            auto_cut: AutoCut::Enabled(1),
            two_colors: false,
            cut_at_end: true,
            high_resolution: false,
            feed: media.get_default_feed_dots(),
            compress: false,
        }
    }

    /// Enable auto cut per
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

    pub fn two_colors(self, two_colors: bool) -> Self {
        Config { two_colors, ..self }
    }

    pub fn compress(self, flag: bool) -> Self {
        Config {
            compress: flag,
            ..self
        }
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
