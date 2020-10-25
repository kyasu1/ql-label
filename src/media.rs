#[derive(Debug)]
pub enum Media {
    Continuous12,
    Continuous29,
    Continuous38,
    Continuous50,
    Continuous54,
    Continuous62,

    DieCut17x54,
    DieCut17x87,
    DieCut23x23,
    DieCut29x42,
    DieCut29x90,
    DieCut38x90,
    DieCut39x48,
    DieCut52x29,
    DieCut60x86,
    DieCut62x29,
    DieCut62x100,
    DieCut12Dia,
    DieCut24Dia,
    DieCut58Dia,
}

struct MediaSize {
    mm: f32,
    dots: u32,
}

struct MediaSpec {
    id: u16,
    width: MediaSize,
    length: Option<MediaSize>,
    margin: MediaSize,
    offset: Option<MediaSize>,
    pins_right: u32,
}

impl MediaSpec {
    fn pins_left(&self) -> u32 {
        720 - self.width.dots - self.pins_right
    }
    fn pins_effective(&self) -> u32 {
        self.width.dots - self.margin.dots * 2
    }
}

impl Media {
    fn spec(&self) -> MediaSpec {
        match self {
            Self::Continuous12 => MediaSpec {
                id: 257,
                width: MediaSize {
                    mm: 12.0,
                    dots: 142,
                },
                length: None,
                margin: MediaSize { mm: 1.5, dots: 18 },
                offset: None,
                pins_right: 29,
            },
            Self::Continuous29 => MediaSpec {
                id: 258,
                width: MediaSize {
                    mm: 29.0,
                    dots: 342,
                },
                length: None,
                margin: MediaSize { mm: 1.5, dots: 18 },
                offset: None,
                pins_right: 6,
            },
            Self::Continuous38 => MediaSpec {
                id: 264,
                width: MediaSize {
                    mm: 38.0,
                    dots: 449,
                },
                length: None,
                margin: MediaSize { mm: 1.5, dots: 18 },
                offset: None,
                pins_right: 12,
            },
            Self::Continuous50 => MediaSpec {
                id: 262,
                width: MediaSize {
                    mm: 50.0,
                    dots: 590,
                },
                length: None,
                margin: MediaSize { mm: 1.5, dots: 18 },
                offset: None,
                pins_right: 12,
            },
            Self::Continuous54 => MediaSpec {
                id: 261,
                width: MediaSize {
                    mm: 54.0,
                    dots: 636,
                },
                length: None,
                margin: MediaSize { mm: 1.9, dots: 23 },
                offset: None,
                pins_right: 0,
            },
            Self::Continuous62 => MediaSpec {
                id: 259,
                width: MediaSize {
                    mm: 62.0,
                    dots: 732,
                },
                length: None,
                margin: MediaSize { mm: 1.5, dots: 18 },
                offset: None,
                pins_right: 12,
            },
            Self::DieCut17x54 => MediaSpec {
                id: 269,
                width: MediaSize {
                    mm: 17.0,
                    dots: 201,
                },
                length: Some(MediaSize {
                    mm: 53.9,
                    dots: 636,
                }),
                margin: MediaSize { mm: 1.5, dots: 18 },
                offset: Some(MediaSize { mm: 3.0, dots: 35 }),
                pins_right: 0,
            },
            _ => MediaSpec {
                id: 257,
                width: MediaSize {
                    mm: 12.0,
                    dots: 142,
                },
                length: None,
                margin: MediaSize { mm: 1.5, dots: 18 },
                offset: None,
                pins_right: 0,
            },
        }
    }

    pub fn from_buf(buf: [u8; 32]) -> Option<Self> {
        let w = buf[10];
        let t = buf[11];
        let l = buf[17];

        match t {
            0x0A => match w {
                // Document says it is 0x4A but actual value seems to be 0x0A
                12 => Some(Self::Continuous12),
                29 => Some(Self::Continuous29),
                38 => Some(Self::Continuous38),
                50 => Some(Self::Continuous50),
                54 => Some(Self::Continuous54),
                62 => Some(Self::Continuous62),
                _ => None,
            },
            0x0B => match (w, l) {
                // Same as above, 0x0B not 0x4B
                (17, 54) => Some(Self::DieCut17x54),
                (17, 87) => Some(Self::DieCut17x87),
                (23, 23) => Some(Self::DieCut23x23),
                (29, 42) => Some(Self::DieCut29x42),
                (29, 90) => Some(Self::DieCut29x90),
                (38, 90) => Some(Self::DieCut38x90),
                (39, 48) => Some(Self::DieCut39x48),
                (52, 29) => Some(Self::DieCut52x29),
                (60, 86) => Some(Self::DieCut60x86),
                (62, 29) => Some(Self::DieCut62x29),
                (62, 100) => Some(Self::DieCut62x100),
                (12, 12) => Some(Self::DieCut12Dia),
                (24, 24) => Some(Self::DieCut24Dia),
                (58, 58) => Some(Self::DieCut58Dia),
                _ => None,
            },
            _ => None,
        }
    }

    fn effective_size(&self) -> (u32, u32) {
        let spec = self.spec();
        (spec.width.dots - spec.margin.dots * 2, 0)
    }
}
