pub enum MediaType {
    Endless,
    DieCut,
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

trait Media {
    fn media_type() -> MediatTYpe;
    fn width() -> Width;
    fn length() -> Length;
}

struct Endless12;

impl Media for Endless12 {
    fn media_type() -> MediaType {
        Endless
    }

    fn width() -> Width {
        width: Width {
            mm: 12,
            left: 585,
            effective: 106,
            right: 29,
        }
    }

    fn length() -> Length {
        length: Length { mm: 0, dots: 0 }
    }
}

struct Endless29;

impl Media for Endless29 {
    fn media_type() -> MediaType {
        Endless
    }
}
