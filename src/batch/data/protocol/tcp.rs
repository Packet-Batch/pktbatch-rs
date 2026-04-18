use serde::de;

use crate::{
    batch::data::protocol::ProtocolExt, config::batch::data::protocol::tcp::TcpOpts as TcpOptsCfg,
};

pub const LEN_TCP_HDR: usize = 20;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TcpOpts {
    pub src_port: Option<u16>,
    pub dst_port: Option<u16>,

    pub flags: u8,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProtocolTcp {
    pub src_port: Option<u16>,
    pub dst_port: Option<u16>,

    pub flags: u8,
}

impl From<TcpOptsCfg> for TcpOpts {
    fn from(cfg: TcpOptsCfg) -> Self {
        Self {
            src_port: cfg.src_port,
            dst_port: cfg.dst_port,
            flags: cfg.flags_to_u8(),
        }
    }
}

impl ProtocolExt for ProtocolTcp {
    type Opts = TcpOpts;

    /// Not used.
    fn new(_proto: &str, opts: Self::Opts) -> Self {
        ProtocolTcp::from(opts)
    }

    fn get_hdr_len(&self) -> usize {
        LEN_TCP_HDR
    }

    fn get_proto_num(&self) -> u8 {
        6
    }

    fn get_src_port(&self) -> Option<u16> {
        self.src_port
    }

    fn get_dst_port(&self) -> Option<u16> {
        self.dst_port
    }
}
