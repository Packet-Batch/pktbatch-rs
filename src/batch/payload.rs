use crate::config::batch::payload::PayloadOpts as PayloadOptsCfg;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PayloadOpts {
    pub len_min: Option<u16>,
    pub len_max: Option<u16>,

    pub is_static: bool,
    pub is_file: bool,
    pub is_string: bool,

    pub exact: Option<String>,
}

impl From<PayloadOptsCfg> for PayloadOpts {
    fn from(cfg: PayloadOptsCfg) -> Self {
        Self {
            len_min: cfg.len_min,
            len_max: cfg.len_max,
            is_static: cfg.is_static,
            is_file: cfg.is_file,
            is_string: cfg.is_string,
            exact: cfg.exact,
        }
    }
}
