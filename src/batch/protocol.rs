use crate::{
    batch::protocol::{icmp::ProtocolIcmp, tcp::ProtocolTcp, udp::ProtocolUdp},
    config::batch::protocol::ProtocolOpts as ProtocolOptsCfg,
};

pub mod icmp;
pub mod tcp;
pub mod udp;

#[derive(Clone, PartialEq, Eq)]
pub enum ProtocolOpts {
    Tcp(ProtocolTcp),
    Udp(ProtocolUdp),
    Icmp(ProtocolIcmp),
}

impl Default for ProtocolOpts {
    fn default() -> Self {
        ProtocolOpts::Tcp(ProtocolTcp::default())
    }
}

impl From<ProtocolOptsCfg> for ProtocolOpts {
    fn from(cfg: ProtocolOptsCfg) -> Self {
        match cfg {
            ProtocolOptsCfg::Tcp(tcp) => ProtocolOpts::Tcp(tcp.into()),
            ProtocolOptsCfg::Udp(udp) => ProtocolOpts::Udp(udp.into()),
            ProtocolOptsCfg::Icmp(icmp) => ProtocolOpts::Icmp(icmp.into()),
        }
    }
}
