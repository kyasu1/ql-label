struct Label {
    id: u8,
    name: String,
    width: u8,
    length: Option<u8>,
    form_factor: FormFactor,
    width_dots: u8,
    length_dots: Option<u8>,
    
}

enum FormFactor {
    Endless,
    DieCut,
    RoundDieCut,
    PtouchEndless,
}