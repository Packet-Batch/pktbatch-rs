use pnet::packet::{tcp::TcpFlags};

use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct TcpOpts {
    pub src_port: Option<u16>,
    pub dst_port: Option<u16>,

    pub flag_syn: bool,
    pub flag_ack: bool,
    pub flag_fin: bool,
    pub flag_rst: bool,
    pub flag_psh: bool,
    pub flag_urg: bool,
    pub flag_ece: bool,
    pub flag_cwr: bool,
}

impl Default for TcpOpts {
    fn default() -> Self {
        TcpOpts {
            src_port: None,
            dst_port: Some(22), // Default to SSH since TCP is default protocol for Packet Batch.
            flag_syn: false,
            flag_ack: false,
            flag_fin: false,
            flag_rst: false,
            flag_psh: false,
            flag_urg: false,
            flag_ece: false,
            flag_cwr: false,
        }
    }
}

impl TcpOpts {
    pub fn flags_to_u8(&self) -> u8 {
        let mut flags = 0;

        if self.flag_syn {
            flags |= TcpFlags::SYN;
        }

        if self.flag_ack {
            flags |= TcpFlags::ACK;
        }

        if self.flag_fin {
            flags |= TcpFlags::FIN;
        }

        if self.flag_rst {
            flags |= TcpFlags::RST;
        }

        if self.flag_psh {
            flags |= TcpFlags::PSH;
        }

        if self.flag_urg {
            flags |= TcpFlags::URG;
        }

        if self.flag_ece {
            flags |= TcpFlags::ECE;
        }

        if self.flag_cwr {
            flags |= TcpFlags::CWR;
        }

        flags
    }
}