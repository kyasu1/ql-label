#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Media {
    Continuous(ContinuousType),
    DieCut(DieCutType),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ContinuousType {
    Continuous12,
    Continuous29,
    Continuous38,
    Continuous50,
    Continuous54,
    Continuous62,
    Continuous62Red,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DieCutType {
    DieCut17x54,
    DieCut17x87,
    DieCut23x23,
    DieCut29x42,
    DieCut29x90,
    DieCut38x90,
    DieCut39x48,
    DieCut52x29,
    DieCut54x29,
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

struct Width {
    mm: u8,
    left: u32,
    effective: u32,
    right: u32,
}

struct Length {
    mm: u8,
    dots: u32,
}

pub struct MediaSpec {
    id: u16,
    width: Width,
    length: Length,
    margin: MediaSize,
    offset: Option<MediaSize>,
}

impl MediaSpec {
    pub fn width_mm(&self) -> u8 {
        self.width.mm
    }
    
    pub fn length_mm(&self) -> u8 {
        self.length.mm
    }
}

impl Media {
    pub fn spec(&self) -> MediaSpec {
        match self {
            Self::Continuous(t) => match t {
                ContinuousType::Continuous12 => MediaSpec {
                    id: 257,
                    width: Width {
                        mm: 12,
                        left: 585,
                        effective: 106,
                        right: 29,
                    },
                    length: Length { mm: 0, dots: 0 },
                    margin: MediaSize { mm: 1.5, dots: 18 },
                    offset: None,
                },
                ContinuousType::Continuous29 => MediaSpec {
                    id: 258,
                    width: Width {
                        mm: 29,
                        left: 408,
                        effective: 306,
                        right: 6,
                    },
                    length: Length { mm: 0, dots: 0 },
                    margin: MediaSize { mm: 1.5, dots: 18 },
                    offset: None,
                },
                ContinuousType::Continuous38 => MediaSpec {
                    id: 264,
                    width: Width {
                        mm: 38,
                        left: 295,
                        effective: 413,
                        right: 12,
                    },
                    length: Length { mm: 0, dots: 0 },
                    margin: MediaSize { mm: 1.5, dots: 18 },
                    offset: None,
                },
                ContinuousType::Continuous50 => MediaSpec {
                    id: 262,
                    width: Width {
                        mm: 50,
                        left: 154,
                        effective: 554,
                        right: 12,
                    },
                    length: Length { mm: 0, dots: 0 },
                    margin: MediaSize { mm: 1.5, dots: 18 },
                    offset: None,
                },
                ContinuousType::Continuous54 => MediaSpec {
                    id: 261,
                    width: Width {
                        mm: 54,
                        left: 130,
                        effective: 590,
                        right: 0,
                    },
                    length: Length { mm: 0, dots: 0 },
                    margin: MediaSize { mm: 1.9, dots: 23 },
                    offset: None,
                },
                ContinuousType::Continuous62 => MediaSpec {
                    id: 259,
                    width: Width {
                        mm: 62,
                        left: 12,
                        effective: 696,
                        right: 12,
                    },
                    length: Length { mm: 0, dots: 0 },
                    margin: MediaSize { mm: 1.5, dots: 18 },
                    offset: None,
                },
                ContinuousType::Continuous62Red => MediaSpec {
                    id: 259,
                    width: Width {
                        mm: 62,
                        left: 12,
                        effective: 696,
                        right: 12,
                    },
                    length: Length { mm: 0, dots: 0 },
                    margin: MediaSize { mm: 1.5, dots: 18 },
                    offset: None,
                },
            },
            Self::DieCut(t) => match t {
                DieCutType::DieCut17x54 => MediaSpec {
                    id: 269,
                    width: Width {
                        mm: 17,
                        left: 555,
                        effective: 165,
                        right: 0,
                    },
                    length: Length { mm: 54, dots: 636 },
                    margin: MediaSize { mm: 1.5, dots: 18 },
                    offset: Some(MediaSize { mm: 3.0, dots: 35 }),
                },
                DieCutType::DieCut17x87 => MediaSpec {
                    id: 270,
                    width: Width {
                        mm: 17,
                        left: 555,
                        effective: 165,
                        right: 0,
                    },
                    length: Length { mm: 87, dots: 1026 },
                    margin: MediaSize { mm: 1.5, dots: 18 },
                    offset: Some(MediaSize { mm: 3.0, dots: 35 }),
                },
                DieCutType::DieCut23x23 => MediaSpec {
                    id: 370,
                    width: Width {
                        mm: 23,
                        left: 442,
                        effective: 236,
                        right: 42,
                    },
                    length: Length { mm: 23, dots: 272 },
                    margin: MediaSize { mm: 1.5, dots: 18 },
                    offset: Some(MediaSize { mm: 3.0, dots: 35 }),
                },
                DieCutType::DieCut29x42 => MediaSpec {
                    id: 358,
                    width: Width {
                        mm: 29,
                        left: 408,
                        effective: 306,
                        right: 6,
                    },
                    length: Length { mm: 42, dots: 495 },
                    margin: MediaSize { mm: 1.5, dots: 18 },
                    offset: Some(MediaSize { mm: 3.0, dots: 35 }),
                },
                DieCutType::DieCut29x90 => MediaSpec {
                    id: 271,
                    width: Width {
                        mm: 29,
                        left: 408,
                        effective: 306,
                        right: 6,
                    },
                    length: Length { mm: 90, dots: 1061 },
                    margin: MediaSize { mm: 1.5, dots: 18 },
                    offset: Some(MediaSize { mm: 3.0, dots: 35 }),
                },
                DieCutType::DieCut38x90 => MediaSpec {
                    id: 272,
                    width: Width {
                        mm: 38,
                        left: 295,
                        effective: 413,
                        right: 12,
                    },
                    length: Length { mm: 90, dots: 1061 },
                    margin: MediaSize { mm: 1.5, dots: 18 },
                    offset: Some(MediaSize { mm: 3.0, dots: 35 }),
                },
                DieCutType::DieCut39x48 => MediaSpec {
                    id: 367,
                    width: Width {
                        mm: 39,
                        left: 289,
                        effective: 425,
                        right: 6,
                    },
                    length: Length { mm: 48, dots: 565 },
                    margin: MediaSize { mm: 1.5, dots: 18 },
                    offset: Some(MediaSize { mm: 3.0, dots: 35 }),
                },
                DieCutType::DieCut52x29 => MediaSpec {
                    id: 374,
                    width: Width {
                        mm: 52,
                        left: 142,
                        effective: 578,
                        right: 0,
                    },
                    length: Length { mm: 29, dots: 341 },
                    margin: MediaSize { mm: 1.5, dots: 18 },
                    offset: Some(MediaSize { mm: 3.0, dots: 35 }),
                },
                DieCutType::DieCut54x29 => MediaSpec {
                    id: 382,
                    width: Width {
                        mm: 54,
                        left: 59,
                        effective: 602,
                        right: 59,
                    },
                    length: Length { mm: 29, dots: 341 },
                    margin: MediaSize { mm: 1.5, dots: 18 },
                    offset: Some(MediaSize { mm: 3.0, dots: 35 }),
                },
                DieCutType::DieCut60x86 => MediaSpec {
                    id: 383,
                    width: Width {
                        mm: 60,
                        left: 24,
                        effective: 672,
                        right: 24,
                    },
                    length: Length { mm: 86, dots: 1024 },
                    margin: MediaSize { mm: 1.5, dots: 18 },
                    offset: Some(MediaSize { mm: 3.0, dots: 35 }),
                },
                DieCutType::DieCut62x29 => MediaSpec {
                    id: 274,
                    width: Width {
                        mm: 62,
                        left: 12,
                        effective: 696,
                        right: 12,
                    },
                    length: Length { mm: 29, dots: 341 },
                    margin: MediaSize { mm: 1.5, dots: 18 },
                    offset: Some(MediaSize { mm: 3.0, dots: 35 }),
                },
                DieCutType::DieCut62x100 => MediaSpec {
                    id: 275,
                    width: Width {
                        mm: 62,
                        left: 12,
                        effective: 696,
                        right: 12,
                    },
                    length: Length {
                        mm: 100,
                        dots: 1179,
                    },
                    margin: MediaSize { mm: 1.5, dots: 18 },
                    offset: Some(MediaSize { mm: 3.0, dots: 35 }),
                },
                DieCutType::DieCut12Dia => MediaSpec {
                    id: 362,
                    width: Width {
                        mm: 12,
                        left: 513,
                        effective: 94,
                        right: 113,
                    },
                    length: Length { mm: 12, dots: 142 },
                    margin: MediaSize { mm: 2.0, dots: 24 },
                    offset: Some(MediaSize { mm: 2.0, dots: 24 }),
                },
                DieCutType::DieCut24Dia => MediaSpec {
                    id: 363,
                    width: Width {
                        mm: 24,
                        left: 442,
                        effective: 236,
                        right: 42,
                    },
                    length: Length { mm: 24, dots: 284 },
                    margin: MediaSize { mm: 2.0, dots: 24 },
                    offset: Some(MediaSize { mm: 2.0, dots: 24 }),
                },
                DieCutType::DieCut58Dia => MediaSpec {
                    id: 273,
                    width: Width {
                        mm: 58,
                        left: 51,
                        effective: 618,
                        right: 51,
                    },
                    length: Length { mm: 58, dots: 686 },
                    margin: MediaSize { mm: 3.0, dots: 35 },
                    offset: Some(MediaSize { mm: 3.0, dots: 35 }),
                },
            },
        }
    }

    pub fn from_id(id: u16) -> Option<Self> {
        match id {
            // Document says it is 0x4A but actual value seems to be 0x0A
            257 => Some(Self::Continuous(ContinuousType::Continuous12)),
            258 => Some(Self::Continuous(ContinuousType::Continuous29)),
            264 => Some(Self::Continuous(ContinuousType::Continuous38)),
            262 => Some(Self::Continuous(ContinuousType::Continuous50)),
            261 => Some(Self::Continuous(ContinuousType::Continuous54)),
            259 => Some(Self::Continuous(ContinuousType::Continuous62)),
            //   0x81 => Some(Self::Continuous(ContinuousType::Continuous62Red)),
            // Same as above, 0x0B not 0x4B
            269 => Some(Self::DieCut(DieCutType::DieCut17x54)),
            270 => Some(Self::DieCut(DieCutType::DieCut17x87)),
            370 => Some(Self::DieCut(DieCutType::DieCut23x23)),
            358 => Some(Self::DieCut(DieCutType::DieCut29x42)),
            271 => Some(Self::DieCut(DieCutType::DieCut29x90)),
            272 => Some(Self::DieCut(DieCutType::DieCut38x90)),
            367 => Some(Self::DieCut(DieCutType::DieCut39x48)),
            374 => Some(Self::DieCut(DieCutType::DieCut52x29)),
            382 => Some(Self::DieCut(DieCutType::DieCut54x29)),
            383 => Some(Self::DieCut(DieCutType::DieCut60x86)),
            274 => Some(Self::DieCut(DieCutType::DieCut62x29)),
            275 => Some(Self::DieCut(DieCutType::DieCut62x100)),
            362 => Some(Self::DieCut(DieCutType::DieCut12Dia)),
            363 => Some(Self::DieCut(DieCutType::DieCut24Dia)),
            273 => Some(Self::DieCut(DieCutType::DieCut58Dia)),
            _ => None,
        }
    }

    pub fn get_default_feed_dots(&self) -> u16 {
        match self {
            Self::Continuous(_) => 35,
            Self::DieCut(_) => 0,
        }
    }

    pub fn check_feed_value(&self, feed: u16) -> Result<[u8; 2], String> {
        match self {
            Self::Continuous(_) => {
                if feed < 35 || feed > 1500 {
                    Err(format!("Feed value {} is out range.", feed))
                } else {
                    Ok(feed.to_le_bytes())
                }
            }
            Self::DieCut(_) => {
                if feed != 0 {
                    Err(format!(
                        "Feed value {} must be zero for die-cut medias",
                        feed
                    ))
                } else {
                    Ok([0x00, 0x00])
                }
            }
        }
    }
    pub fn set_media(&self, buf: &mut Vec<u8>, qualiy: bool) {
        let qualiry: u8 = if qualiy { 0b01000000 } else { 0b00000000 };
        let spec = self.spec();
        match self {
            Self::Continuous(_) => {
                buf.push(0x86 | qualiry);
                buf.push(0x0A);
            }
            Self::DieCut(_) => {
                buf.push(0x8E | qualiry);
                buf.push(0x0B);
            }
        }
        buf.push(spec.width.mm);
        buf.push(spec.length.mm);
    }

    pub fn from_buf(buf: [u8; 32]) -> Option<Self> {
        let w = buf[10];
        let t = buf[11];
        let l = buf[17];
        let c = buf[25];

        match t {
            0x0A => match w {
                // Document says it is 0x4A but actual value seems to be 0x0A
                12 => Some(Self::Continuous(ContinuousType::Continuous12)),
                29 => Some(Self::Continuous(ContinuousType::Continuous29)),
                38 => Some(Self::Continuous(ContinuousType::Continuous38)),
                50 => Some(Self::Continuous(ContinuousType::Continuous50)),
                54 => Some(Self::Continuous(ContinuousType::Continuous54)),
                62 => match c {
                    0x01 => Some(Self::Continuous(ContinuousType::Continuous62)),
                    0x81 => Some(Self::Continuous(ContinuousType::Continuous62Red)),
                    _ => None,
                },
                _ => None,
            },
            0x0B => match (w, l) {
                // Same as above, 0x0B not 0x4B
                (17, 54) => Some(Self::DieCut(DieCutType::DieCut17x54)),
                (17, 87) => Some(Self::DieCut(DieCutType::DieCut17x87)),
                (23, 23) => Some(Self::DieCut(DieCutType::DieCut23x23)),
                (29, 42) => Some(Self::DieCut(DieCutType::DieCut29x42)),
                (29, 90) => Some(Self::DieCut(DieCutType::DieCut29x90)),
                (38, 90) => Some(Self::DieCut(DieCutType::DieCut38x90)),
                (39, 48) => Some(Self::DieCut(DieCutType::DieCut39x48)),
                (52, 29) => Some(Self::DieCut(DieCutType::DieCut52x29)),
                (54, 29) => Some(Self::DieCut(DieCutType::DieCut54x29)),
                (60, 86) => Some(Self::DieCut(DieCutType::DieCut60x86)),
                (62, 29) => Some(Self::DieCut(DieCutType::DieCut62x29)),
                (62, 100) => Some(Self::DieCut(DieCutType::DieCut62x100)),
                (12, 12) => Some(Self::DieCut(DieCutType::DieCut12Dia)),
                (24, 24) => Some(Self::DieCut(DieCutType::DieCut24Dia)),
                (58, 58) => Some(Self::DieCut(DieCutType::DieCut58Dia)),
                _ => None,
            },
            _ => None,
        }
    }
}
