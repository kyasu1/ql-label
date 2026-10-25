use std::str::FromStr;

#[derive(Debug, Clone, Copy)]
pub enum Model {
    QL500, // TESTED
    QL550,
    QL560,
    QL570,
    QL580N,
    QL600,
    QL650TD,
    QL700,
    QL710W,
    QL720NW, // TESTED
    QL800,   // TESTED
    QL810W,
    QL820NWB, //TESTED
    QL1050,
    QL1060N,
    QL1100,
    QL1110NWB,
    QL1115NWB,
}

#[derive(Debug, PartialEq, Eq)]
pub struct InvalidPrinterName;

impl FromStr for Model {
    type Err = InvalidPrinterName;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "QL-500" => Ok(Self::QL500),
            "QL-720NW" => Ok(Self::QL720NW),
            "QL-800" => Ok(Self::QL800),
            "QL-820NW" => Ok(Self::QL820NWB),
            "QL-820NWB" => Ok(Self::QL820NWB),
            _ => Err(InvalidPrinterName),
        }
    }
}
impl Model {
    pub fn from_code(code: u8) -> Self {
        match code {
            0x4F => Self::QL500, // Also reported by QL-550
            0x47 => Self::QL600,
            0x37 => Self::QL720NW,
            0x38 => Self::QL800,
            0x39 => Self::QL810W,
            0x41 => Self::QL820NWB,
            0x43 => Self::QL1100,
            0x44 => Self::QL1110NWB,
            0x45 => Self::QL1115NWB,
            _ => panic!("Unknown model code {}", code),
        }
    }

    pub fn pid(&self) -> u16 {
        match self {
            Self::QL500 => 0x2015,
            Self::QL600 => 0x20C0,
            Self::QL720NW => 0x2044,
            Self::QL800 => 0x209b,
            Self::QL810W => 0x209c,
            Self::QL820NWB => 0x209d,
            Self::QL1100 => 0x20A7,
            Self::QL1110NWB => 0x20A8,
            Self::QL1115NWB => 0x20AB,
            _ => 0x0000,
        }
    }

    pub fn pins(&self) -> u32 {
        match self {
            Self::QL1050 => crate::WIDE_PRINTER_WIDTH,
            Self::QL1060N => crate::WIDE_PRINTER_WIDTH,
            Self::QL1100 => crate::WIDE_PRINTER_WIDTH,
            Self::QL1110NWB => crate::WIDE_PRINTER_WIDTH,
            Self::QL1115NWB => crate::WIDE_PRINTER_WIDTH,
            _ => crate::NORMAL_PRINTER_WIDTH,
        }
    }

    /// Whether the printer has an auto cutter (`ESC i M` / `ESC i A` commands).
    pub fn has_cutter(&self) -> bool {
        !matches!(self, Self::QL500)
    }

    /// Whether the printer accepts `ESC i a` (switch to raster mode) and
    /// `ESC i !` (automatic status notification). Older models are always in raster mode.
    pub fn supports_mode_setting(&self) -> bool {
        !matches!(self, Self::QL500)
    }

    /// Whether the printer accepts `ESC i K` (expanded mode: cut at end, high resolution, two colors).
    pub fn supports_expanded_mode(&self) -> bool {
        !matches!(self, Self::QL500)
    }

    /// Whether the printer accepts PackBits compressed raster lines (`M 0x02`).
    pub fn supports_compression(&self) -> bool {
        !matches!(self, Self::QL500 | Self::QL800)
    }

    // pub fn supported_medias(&self) -> Vec<Media> {
    //     match self {
    //         Self::QL800 => vec![Media::Continuous29],
    //         Self::QL810W => vec![Media::Continuous29],
    //         Self::QL820NWB => vec![Media::Continuous29],
    //     }
    // }
}
