use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct IpOpts {
    pub src: Option<String>,
    pub srcs: Option<Vec<String>>,

    pub dst: Option<String>,

    pub tos: Option<u8>,

    pub ttl_min: Option<u8>,
    pub ttl_max: Option<u8>,

    pub id_min: Option<u16>,
    pub id_max: Option<u16>,

    pub do_csum: bool,
}

impl Default for IpOpts {
    fn default() -> Self {
        IpOpts {
            src: None,
            srcs: None,
            dst: None,
            tos: None,
            ttl_min: Some(64),
            ttl_max: Some(64),
            id_min: None,
            id_max: None,
            do_csum: true,
        }
    }
}