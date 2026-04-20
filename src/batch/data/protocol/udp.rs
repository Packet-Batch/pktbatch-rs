use anyhow::{Result, anyhow};
use pnet::packet::{
    ipv4::MutableIpv4Packet,
    udp::{MutableUdpPacket, ipv4_checksum},
};

use crate::{
    batch::data::{
        ip::IP_HDR_LEN,
        protocol::{FILL_FLAG_DST_PORT, FILL_FLAG_SRC_PORT, Protocol, ProtocolExt},
    },
    config::batch::data::protocol::udp::UdpOpts as UdpOptsCfg,
    util::rand_num,
};

pub const LEN_UDP_HDR: usize = 8;

pub const FILL_FLAG_UDP_LEN: u32 = 1 << 2;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UdpOpts {
    pub src_port: Option<u16>,
    pub dst_port: Option<u16>,

    pub do_csum: bool,
}

impl From<UdpOptsCfg> for UdpOpts {
    fn from(cfg: UdpOptsCfg) -> Self {
        Self {
            src_port: cfg.src_port,
            dst_port: cfg.dst_port,
            do_csum: cfg.do_csum,
        }
    }
}

impl From<UdpOpts> for Protocol {
    fn from(opts: UdpOpts) -> Self {
        Protocol::Udp(opts)
    }
}

impl ProtocolExt for UdpOpts {
    type Opts = UdpOpts;
    type State = (); // Not used for UDP

    #[inline(always)]
    fn get_hdr_len(&self) -> usize {
        LEN_UDP_HDR
    }

    #[inline(always)]
    fn get_proto_num(&self) -> u8 {
        17
    }

    #[inline(always)]
    fn get_src_port(&self) -> Option<u16> {
        self.src_port
    }

    #[inline(always)]
    fn gen_src_port(&self, buff: &mut [u8], seed: &mut u64) -> Result<(Option<u16>, bool)> {
        // We need to initialize the UDP header so we can set the port.
        let mut udph = match MutableUdpPacket::new(buff) {
            Some(p) => p,
            None => {
                return Err(anyhow!(
                    "Failed to create mutable UDP packet for source port generation"
                ));
            }
        };

        let (port, is_static) = match self.src_port {
            Some(port) => (Some(port), true),
            None => (Some(rand_num(seed, 1, u16::MAX as u64) as u16), false),
        };

        // Set the source port in the UDP header
        if let Some(port) = port {
            udph.set_source(port);
        }

        Ok((port, is_static))
    }

    fn get_dst_port(&self) -> Option<u16> {
        self.dst_port
    }

    fn gen_dst_port(&self, buff: &mut [u8], seed: &mut u64) -> Result<(Option<u16>, bool)> {
        // Initialize UDP header so we can set the port.
        let mut udph = match MutableUdpPacket::new(buff) {
            Some(p) => p,
            None => {
                return Err(anyhow!(
                    "Failed to create mutable UDP packet for destination port generation"
                ));
            }
        };

        let (port, is_static) = match self.dst_port {
            Some(port) => (Some(port), true),
            None => (Some(rand_num(seed, 1, u16::MAX as u64) as u16), false),
        };

        // Set the destination port in the UDP header
        if let Some(port) = port {
            udph.set_destination(port);
        }

        Ok((port, is_static))
    }

    fn gen_checksum(&self, buff: &mut [u8]) -> Result<()> {
        // The buffer should start at the IP header so we can retrieve the source and destination IP addresses.
        let iph = match MutableIpv4Packet::new(buff[..IP_HDR_LEN].as_mut()) {
            Some(p) => p,
            None => {
                return Err(anyhow!(
                    "Failed to create mutable IPv4 packet for UDP checksum calculation"
                ));
            }
        };

        let src_ip = iph.get_source();
        let dst_ip = iph.get_destination();

        let mut udph: MutableUdpPacket<'_> =
            match MutableUdpPacket::new(buff[IP_HDR_LEN..].as_mut()) {
                Some(p) => p,
                None => {
                    return Err(anyhow!(
                        "Failed to create mutable UDP packet for checksum calculation"
                    ));
                }
            };

        udph.set_checksum(
            ipv4_checksum(&udph.to_immutable(), &src_ip, &dst_ip)
                .try_into()
                .map_err(|e| anyhow!("Failed to convert checksum to u16: {}", e))?,
        );

        Ok(())
    }

    #[inline(always)]
    fn fill_init(&self, buff: &mut [u8], seed: &mut u64) -> Result<(bool, bool)> {
        // Buffer length should represent UDP header + payload length.
        let buff_len = { buff.len() };

        {
            let mut udph = match MutableUdpPacket::new(buff) {
                Some(p) => p,
                None => {
                    return Err(anyhow!(
                        "Failed to create mutable UDP packet for filling fields"
                    ));
                }
            };

            // Set UDP header length.
            udph.set_length(buff_len as u16);
        }

        let (is_static_src, is_static_dst) = {
            // Generate both source and destination ports.
            let (_, src_static) = self
                .gen_src_port(buff, seed)
                .map_err(|e| anyhow!("Failed to generate source port: {}", e))?;

            let (_, dst_static) = self
                .gen_dst_port(buff, seed)
                .map_err(|e| anyhow!("Failed to generate destination port: {}", e))?;

            (src_static, dst_static)
        };

        Ok((is_static_src, is_static_dst))
    }

    #[inline(always)]
    fn fill(&self, buff: &mut [u8], flags: u32, seed: &mut u64) -> Result<()> {
        let buff_len = { buff.len() };

        {
            // Regenerate ports if they were not static.
            if (flags & FILL_FLAG_SRC_PORT != 0) && self.src_port.is_none() {
                self.gen_src_port(buff, seed)
                    .map_err(|e| anyhow!("Failed to generate source port: {}", e))?;
            }

            if (flags & FILL_FLAG_DST_PORT != 0) && self.dst_port.is_none() {
                self.gen_dst_port(buff, seed)
                    .map_err(|e| anyhow!("Failed to generate destination port: {}", e))?;
            }
        }

        {
            let mut udph = match MutableUdpPacket::new(buff) {
                Some(p) => p,
                None => {
                    return Err(anyhow!(
                        "Failed to create mutable UDP packet for filling fields"
                    ));
                }
            };

            // Recalculate length if needed.
            if flags & FILL_FLAG_UDP_LEN != 0 {
                udph.set_length(buff_len as u16);
            }
        }

        Ok(())
    }
}
