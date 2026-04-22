use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PayloadOpts {
    pub len_min: Option<u16>,
    pub len_max: Option<u16>,

    pub is_static: bool,
    pub is_file: bool,
    pub is_string: bool,

    pub exact: Option<String>,
}

impl Default for PayloadOpts {
    fn default() -> Self {
        PayloadOpts {
            len_min: None,
            len_max: None,

            is_static: true,
            is_file: false,
            is_string: false,

            exact: None,
        }
    }
}
