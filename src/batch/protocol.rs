use crate::batch::protocol::{icmp::ProtocolIcmp, tcp::ProtocolTcp, udp::ProtocolUdp};

pub mod tcp;
pub mod udp;
pub mod icmp;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Protocol {
    Tcp(ProtocolTcp),
    Udp(ProtocolUdp),
    Icmp(ProtocolIcmp),
}