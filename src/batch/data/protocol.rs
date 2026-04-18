pub mod icmp;
pub mod tcp;
pub mod udp;

use crate::{
    batch::data::protocol::{icmp::ProtocolIcmp, tcp::ProtocolTcp, udp::ProtocolUdp},
    config::batch::data::protocol::ProtocolOpts as ProtocolOptsCfg,
};

pub trait ProtocolExt {
    type Opts;

    /// Creates a new protocol from the given options. The `proto_str` is used to determine which protocol struct to create (e.g., TCP, UDP, ICMP), and the `opts` are the specific options for that protocol.
    /// # Arguments
    /// * `proto_str` - The protocol name as a string (e.g., "tcp", "udp", "icmp"). This is used to determine which protocol struct to create.
    /// * `opts` - The options specific to the protocol (e.g., source/destination ports for TCP/UDP, type/code for ICMP).
    /// # Returns
    /// A new instance of the protocol with the specified options.
    fn new(proto_str: &str, opts: Self::Opts) -> Self
    where
        Self: Sized;

    /// Retrieves the header length for the protocol. This is used for calculating offsets when constructing packets.
    ///
    /// # Returns
    /// The header length in bytes for the protocol.
    fn get_hdr_len(&self) -> usize;

    /// Retrieves the protocol number for the protocol. This is used for setting the correct protocol field in the IP header.
    ///
    /// # Returns
    /// The protocol number as defined by IANA (e.g., TCP=6, UDP=17, ICMP=1).
    fn get_proto_num(&self) -> u8;

    /// Retrieves the source port for the protocol, if applicable. This is used for setting the source port in the transport header.
    ///
    /// # Returns
    /// `Some(u16)` containing the source port for TCP/UDP, or `None` for ICMP.
    fn get_src_port(&self) -> Option<u16>;

    /// Retrieves the destination port for the protocol, if applicable. This is used for setting the destination port in the transport header.
    ///
    /// # Returns
    /// `Some(u16)` containing the destination port for TCP/UDP, or `None` for ICMP.
    fn get_dst_port(&self) -> Option<u16>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Protocol {
    Tcp(ProtocolTcp),
    Udp(ProtocolUdp),
    Icmp(ProtocolIcmp),
}

impl Default for Protocol {
    fn default() -> Self {
        Protocol::Tcp(ProtocolTcp::default())
    }
}

impl From<ProtocolOptsCfg> for Protocol {
    fn from(cfg: ProtocolOptsCfg) -> Self {
        match cfg {
            ProtocolOptsCfg::Tcp(tcp) => Protocol::Tcp(ProtocolTcp::from(tcp)),
            ProtocolOptsCfg::Udp(udp) => Protocol::Udp(udp.into()),
            ProtocolOptsCfg::Icmp(icmp) => Protocol::Icmp(icmp.into()),
        }
    }
}

impl ProtocolExt for Protocol {
    type Opts = ();

    fn new(proto_str: &str, opts: Self::Opts) -> Result<Self> {
        match proto_str {
            "tcp" => Ok(Protocol::Tcp(TcpOpts::from(opts))),
            "udp" => Ok(Protocol::Udp(UdpOpts::from(opts))),
            "icmp" => Ok(Protocol::Icmp(IcmpOpts::from(opts))),
            _ => Err(anyhow!("Unsupported protocol: {}", proto_str)),
        }
    }

    fn get_hdr_len(&self) -> usize {
        match self {
            Protocol::Tcp(tcp) => LEN_TCP_HDR,
            Protocol::Udp(udp) => LEN_UDP_HDR,
            Protocol::Icmp(icmp) => LEN_ICMP_HDR,
        }
    }

    fn get_proto_num(&self) -> u8 {
        match self {
            Protocol::Tcp(_) => 6,
            Protocol::Udp(_) => 17,
            Protocol::Icmp(_) => 1,
        }
    }

    fn get_src_port(&self) -> Option<u16> {
        match self {
            Protocol::Tcp(tcp) => tcp.src_port,
            Protocol::Udp(udp) => udp.src_port,
            Protocol::Icmp(_) => None,
        }
    }

    fn get_dst_port(&self) -> Option<u16> {
        match self {
            Protocol::Tcp(tcp) => tcp.dst_port,
            Protocol::Udp(udp) => udp.dst_port,
            Protocol::Icmp(_) => None,
        }
    }
}
