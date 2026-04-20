pub mod icmp;
pub mod tcp;
pub mod udp;

use anyhow::{Result, anyhow};

use crate::{
    batch::data::protocol::{icmp::IcmpOpts, tcp::TcpOpts, udp::UdpOpts},
    config::batch::data::protocol::ProtocolOpts as ProtocolOptsCfg,
};

pub const FILL_FLAG_SRC_PORT: u32 = 1 << 0;
pub const FILL_FLAG_DST_PORT: u32 = 1 << 1;

pub trait ProtocolExt {
    type Opts;
    type State;

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

    /// Generates the next source port and fills out the buffer.
    ///
    /// # Arguments
    /// * `buff` - The packet buffer starting at the transport header.
    /// * `seed` - A mutable reference to a seed value for generating random ports if needed.
    ///
    /// # Returns
    /// Some(u16, bool) containing the next source port for TCP/UDP and a boolean indicating whether the port is static (true) or random (false), or `None` for ICMP.
    fn gen_src_port(&self, buff: &mut [u8], seed: &mut u64) -> Result<(Option<u16>, bool)>;

    /// Retrieves the destination port for the protocol, if applicable. This is used for setting the destination port in the transport header.
    ///
    /// # Returns
    /// `Some(u16)` containing the destination port for TCP/UDP, or `None` for ICMP.
    fn get_dst_port(&self) -> Option<u16>;

    /// Generates the next destination port and fills out the buffer.
    ///
    /// # Arguments
    /// * `buff` - The packet buffer starting at the transport header.
    /// * `seed` - A mutable reference to a seed value for generating random ports if needed.
    ///
    /// # Returns
    /// Some(u16, bool) containing the next destination port for TCP/UDP and a boolean indicating whether the port is static (true) or random (false), or `None` for ICMP.
    fn gen_dst_port(&self, buff: &mut [u8], seed: &mut u64) -> Result<(Option<u16>, bool)>;

    /// Generates checksum for the protocol header, if applicable. This is used for calculating the checksum field in the transport header.
    ///
    /// # Arguments
    /// * `buff` - A mutable reference to the packet buffer where the checksum should be calculated and set.
    ///
    /// # Returns
    /// A `Result` indicating success or failure of checksum generation. Errors may occur if the packet buffer is not properly formatted or if the protocol does not support checksums.
    fn gen_checksum(&self, buff: &mut [u8]) -> Result<()>;

    /// Fills the protocol-specific fields within the header. This is the initial filling before the thread packet generation loop. Used for static fields.
    ///
    /// # Arguments
    /// * `buff` - The packet buffer starting at the transport header.
    /// * `seed` - A mutable reference to a seed value for generating random fields if needed.
    ///
    /// # Returns
    /// A `Result` indicating success or failure of filling the header fields along with whether the source and destination ports are static. Errors may occur if the packet buffer is not properly formatted or if there is an issue with generating random values for the fields.
    fn fill_init(&self, buff: &mut [u8], seed: &mut u64) -> Result<(bool, bool)>;

    /// Called when refilling the packet buffer for the next packet generation. Used for dynamic fields that need to be updated for each packet.
    ///
    /// # Arguments
    /// * `buff` - The packet buffer starting at the transport header.
    /// * `flags` - A bitmask of flags that may indicate which fields need to be refilled.
    /// * `seed` - A mutable reference to a seed value for generating random fields if needed.
    ///
    /// # Returns
    /// A `Result` indicating success or failure of refilling the header fields. Errors may occur if the packet buffer is not properly formatted or if there is an issue with generating random values for the fields.
    fn fill(&self, buff: &mut [u8], flags: u32, seed: &mut u64) -> Result<()>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Protocol {
    Tcp(TcpOpts),
    Udp(UdpOpts),
    Icmp(IcmpOpts),
}

impl Default for Protocol {
    fn default() -> Self {
        Protocol::Tcp(TcpOpts::default())
    }
}

impl From<ProtocolOptsCfg> for Protocol {
    fn from(cfg: ProtocolOptsCfg) -> Self {
        match cfg {
            ProtocolOptsCfg::Tcp(tcp) => Protocol::Tcp(tcp.into()),
            ProtocolOptsCfg::Udp(udp) => Protocol::Udp(udp.into()),
            ProtocolOptsCfg::Icmp(icmp) => Protocol::Icmp(icmp.into()),
        }
    }
}

impl ProtocolExt for Protocol {
    type Opts = ();
    type State = ();

    fn get_hdr_len(&self) -> usize {
        return match self {
            Protocol::Tcp(tcp) => tcp.get_hdr_len(),
            Protocol::Udp(udp) => udp.get_hdr_len(),
            Protocol::Icmp(icmp) => icmp.get_hdr_len(),
        };
    }

    fn get_proto_num(&self) -> u8 {
        match self {
            Protocol::Tcp(tcp) => tcp.get_proto_num(),
            Protocol::Udp(udp) => udp.get_proto_num(),
            Protocol::Icmp(icmp) => icmp.get_proto_num(),
        }
    }

    fn get_src_port(&self) -> Option<u16> {
        match self {
            Protocol::Tcp(tcp) => tcp.src_port,
            Protocol::Udp(udp) => udp.src_port,
            Protocol::Icmp(_) => None,
        }
    }

    fn gen_src_port(&self, buff: &mut [u8], seed: &mut u64) -> Result<(Option<u16>, bool)> {
        match self {
            Protocol::Tcp(tcp) => tcp.gen_src_port(buff, seed),
            Protocol::Udp(udp) => udp.gen_src_port(buff, seed),
            Protocol::Icmp(_) => Ok((None, false)),
        }
    }

    fn get_dst_port(&self) -> Option<u16> {
        match self {
            Protocol::Tcp(tcp) => tcp.dst_port,
            Protocol::Udp(udp) => udp.dst_port,
            Protocol::Icmp(_) => None,
        }
    }

    fn gen_dst_port(&self, buff: &mut [u8], seed: &mut u64) -> Result<(Option<u16>, bool)> {
        match self {
            Protocol::Tcp(tcp) => tcp.gen_dst_port(buff, seed),
            Protocol::Udp(udp) => udp.gen_dst_port(buff, seed),
            Protocol::Icmp(_) => Ok((None, false)),
        }
    }

    fn gen_checksum(&self, buff: &mut [u8]) -> Result<()> {
        match self {
            Protocol::Tcp(tcp) => tcp.gen_checksum(buff),
            Protocol::Udp(udp) => udp.gen_checksum(buff),
            Protocol::Icmp(icmp) => icmp.gen_checksum(buff),
        }
    }

    fn fill_init(&self, buff: &mut [u8], seed: &mut u64) -> Result<(bool, bool)> {
        match self {
            Protocol::Tcp(tcp) => tcp.fill_init(buff, seed),
            Protocol::Udp(udp) => udp.fill_init(buff, seed),
            Protocol::Icmp(icmp) => icmp.fill_init(buff, seed),
        }
    }

    fn fill(&self, buff: &mut [u8], flags: u32, seed: &mut u64) -> Result<()> {
        match self {
            Protocol::Tcp(tcp) => tcp.fill(buff, flags, seed),
            Protocol::Udp(udp) => udp.fill(buff, flags, seed),
            Protocol::Icmp(icmp) => icmp.fill(buff, flags, seed),
        }
    }
}

impl Protocol {
    /// Factory method to create a Protocol instance from a protocol string and options.
    ///
    /// # Arguments
    /// * `proto_str` - A string representing the protocol type ("tcp", "udp", "icmp").
    /// * `opts` - An options struct that can be converted into a Protocol variant (e.g., TcpOpts, UdpOpts, IcmpOpts).
    ///
    /// # Returns
    /// A `Result` containing the created `Protocol` instance if successful, or an `anyhow::Error` if the protocol string does not match the provided options.
    pub fn new(proto_str: &str, opts: impl Into<Protocol>) -> Result<Self> {
        let proto = opts.into();

        match (proto_str, &proto) {
            ("tcp", Protocol::Tcp(_)) => Ok(proto),
            ("udp", Protocol::Udp(_)) => Ok(proto),
            ("icmp", Protocol::Icmp(_)) => Ok(proto),
            _ => Err(anyhow!("Mismatched protocol: expected {}", proto_str)),
        }
    }
}
