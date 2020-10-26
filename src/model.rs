use crate::media::Media;

#[derive(Debug, Clone, Copy)]
pub enum Model {
    QL800,
    QL810W,
    QL820NWB,
}

impl Model {
    pub fn from_code(code: u8) -> Self {
        match code {
            0x38 => (Self::QL800),
            0x39 => (Self::QL810W),
            0x41 => (Self::QL820NWB),
            _ => panic!("Unknown model code {}", code),
        }
    }

    pub fn pid(&self) -> u16 {
        match self {
            Self::QL800 => 0x209b,
            Self::QL810W => 0x209c,
            Self::QL820NWB => 0x209d,
        }
    }

    // pub fn supported_medias(&self) -> Vec<Media> {
    //     match self {
    //         Self::QL800 => vec![Media::Continuous29],
    //         Self::QL810W => vec![Media::Continuous29],
    //         Self::QL820NWB => vec![Media::Continuous29],
    //     }
    // }
}
