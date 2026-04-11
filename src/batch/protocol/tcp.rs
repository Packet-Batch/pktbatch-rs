use serde::{Deserialize, Serialize};

use crate::config::batch::protocol::tcp::TcpOpts;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
enum TcpFlags {
    // These aren't the real TCP flag values!! Just for configuration
    SYN = 1 << 0,
    ACK = 1 << 1,
    FIN = 1 << 2,
    RST = 1 << 3,
    PSH = 1 << 4,
    URG = 1 << 5,
    ECE = 1 << 6,
    CWR = 1 << 7,
}

#[derive(Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolTcp {
    pub src_port: Option<u16>,
    pub dst_port: Option<u16>,

    pub flags: u8,
}

impl From<TcpOpts> for ProtocolTcp {
    fn from(cfg: TcpOpts) -> Self {
        Self {
            src_port: cfg.src_port,
            dst_port: cfg.dst_port,
            flags: cfg.flags_to_u8(),
        }
    }
}
