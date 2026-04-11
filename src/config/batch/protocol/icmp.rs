use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct IcmpOpts {
    pub icmp_type: Option<u8>,
    pub icmp_code: Option<u8>,
}

impl Default for IcmpOpts {
    fn default() -> Self {
        IcmpOpts {
            icmp_type: Some(8),
            icmp_code: None,
        }
    }
}