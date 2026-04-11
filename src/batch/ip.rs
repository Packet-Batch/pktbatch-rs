use crate::batch::ip::source::IpSource;

pub mod source;

#[derive(Clone, PartialEq, Eq)]
pub struct BatchIp {
    pub src: Option<IpSource>,
    pub dst: String,

    pub tos: Option<u8>,
    
    pub ttl_min: Option<u8>,
    pub ttl_max: Option<u8>,

    pub id_min: Option<u16>,
    pub id_max: Option<u16>,

    pub do_csum: bool,
}

impl Default for BatchIp {
    fn default() -> Self {
        BatchIp {
            src: None,
            dst: "".to_string(),
            tos: None,
            ttl_min: None,
            ttl_max: None,
            id_min: None,
            id_max: None,
            do_csum: false,
        }
    }
}

