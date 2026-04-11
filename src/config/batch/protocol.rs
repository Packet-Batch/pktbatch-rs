pub mod icmp;
pub mod tcp;
pub mod udp;

use serde::{Deserialize, Serialize};

use crate::batch::protocol::{icmp::ProtocolIcmp, tcp::ProtocolTcp, udp::ProtocolUdp};

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", content = "opts")]
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
