use serde::{Deserialize, Serialize};

use crate::config::batch::protocol::udp::UdpOpts;

#[derive(Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolUdp {
    pub src_port: Option<u16>,
    pub dst_port: Option<u16>,
}

impl From<UdpOpts> for ProtocolUdp {
    fn from(cfg: UdpOpts) -> Self {
        Self {
            src_port: cfg.src_port,
            dst_port: cfg.dst_port,
        }
    }
}
