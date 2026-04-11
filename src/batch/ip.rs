use crate::config::batch::ip::IpOpts as IpOptsCfg;

pub mod source;

#[derive(Clone, PartialEq, Eq)]
pub struct IpOpts {
    pub src: Option<Vec<String>>,

    pub dst: String,

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

impl From<IpOptsCfg> for IpOpts {
    fn from(cfg: IpOptsCfg) -> Self {
        Self {
            src: cfg.srcs.or_else(|| cfg.src.map(|s| vec![s])),
            dst: cfg.dst.unwrap_or_default(),
            tos: cfg.tos,
            ttl_min: cfg.ttl_min,
            ttl_max: cfg.ttl_max,
            id_min: cfg.id_min,
            id_max: cfg.id_max,
            do_csum: cfg.do_csum,
        }
    }
}
