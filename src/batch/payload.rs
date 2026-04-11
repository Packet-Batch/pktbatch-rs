#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Payload {
    pub len_min: Option<u16>,
    pub len_max: Option<u16>,

    pub is_static: bool,
    pub is_file: bool,
    pub is_string: bool,

    pub exact: Option<String>,
}