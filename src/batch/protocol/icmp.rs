use serde::{Deserialize, Serialize};

use crate::config::batch::protocol::icmp::IcmpOpts;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolIcmp {
    pub icmp_type: u8,
    pub icmp_code: u8,
}

impl Default for ProtocolIcmp {
    fn default() -> Self {
        ProtocolIcmp {
            icmp_type: 8, // Echo Request
            icmp_code: 0,
        }
    }
}

impl From<IcmpOpts> for ProtocolIcmp {
    fn from(cfg: IcmpOpts) -> Self {
        Self {
            icmp_type: cfg.icmp_type.unwrap_or_default(),
            icmp_code: cfg.icmp_code.unwrap_or_default(),
        }
    }
}
