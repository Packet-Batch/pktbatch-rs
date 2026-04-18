use crate::{
    batch::data::protocol::ProtocolExt,
    config::batch::data::protocol::icmp::IcmpOpts as IcmpOptsCfg,
};

pub const LEN_ICMP_HDR: usize = 8;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IcmpOpts {
    pub icmp_type: u8,
    pub icmp_code: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

impl From<IcmpOptsCfg> for IcmpOpts {
    fn from(cfg: IcmpOptsCfg) -> Self {
        Self {
            icmp_type: cfg.icmp_type.unwrap_or_default(),
            icmp_code: cfg.icmp_code.unwrap_or_default(),
        }
    }
}

impl ProtocolExt for ProtocolIcmp {
    type Opts = IcmpOpts;

    fn new(_proto: &str, opts: Self::Opts) -> Self {
        ProtocolIcmp::from(opts)
    }

    fn get_hdr_len(&self) -> usize {
        LEN_ICMP_HDR
    }

    fn get_proto_num(&self) -> u8 {
        1
    }

    fn get_src_port(&self) -> Option<u16> {
        None
    }

    fn get_dst_port(&self) -> Option<u16> {
        None
    }
}
