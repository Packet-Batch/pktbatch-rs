use crate::{
    batch::data::protocol::ProtocolExt, config::batch::data::protocol::udp::UdpOpts as UdpOptsCfg,
};

pub const LEN_UDP_HDR: usize = 8;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UdpOpts {
    pub src_port: Option<u16>,
    pub dst_port: Option<u16>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProtocolUdp {
    pub src_port: Option<u16>,
    pub dst_port: Option<u16>,
}

impl From<UdpOptsCfg> for UdpOpts {
    fn from(cfg: UdpOptsCfg) -> Self {
        Self {
            src_port: cfg.src_port,
            dst_port: cfg.dst_port,
        }
    }
}

impl ProtocolExt for ProtocolUdp {
    type Opts = UdpOpts;

    fn new(_proto: &str, opts: Self::Opts) -> Self {
        ProtocolUdp::from(opts)
    }

    fn get_hdr_len(&self) -> usize {
        LEN_UDP_HDR
    }

    fn get_proto_num(&self) -> u8 {
        17
    }

    fn get_src_port(&self) -> Option<u16> {
        self.src_port
    }

    fn get_dst_port(&self) -> Option<u16> {
        self.dst_port
    }
}
