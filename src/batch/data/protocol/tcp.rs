use anyhow::{Result, anyhow};
use pnet::packet::{
    ipv4::MutableIpv4Packet,
    tcp::{MutableTcpPacket, ipv4_checksum},
};

use crate::{
    batch::data::{
        ip::IP_HDR_LEN,
        protocol::{FILL_FLAG_DST_PORT, FILL_FLAG_SRC_PORT, Protocol, ProtocolExt},
    },
    config::batch::data::protocol::tcp::TcpOpts as TcpOptsCfg,
    util::rand_num,
};

pub const TCP_DATA_OFF: usize = 5; // used for a 20-byte TCP header (no additional options for now)
pub const LEN_TCP_HDR: usize = 5 * 4;

pub const FILL_FLAG_TCP_SRC: u32 = 1 << 0;
pub const FILL_FLAG_TCP_DST: u32 = 1 << 1;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TcpOpts {
    pub src_port: Option<u16>,
    pub dst_port: Option<u16>,

    pub flags: u8,

    pub do_csum: bool,
}

impl From<TcpOptsCfg> for TcpOpts {
    fn from(cfg: TcpOptsCfg) -> Self {
        Self {
            src_port: cfg.src_port,
            dst_port: cfg.dst_port,
            flags: cfg.flags_to_u8(),
            do_csum: cfg.do_csum,
        }
    }
}

impl From<TcpOpts> for Protocol {
    fn from(opts: TcpOpts) -> Self {
        Protocol::Tcp(opts)
    }
}

impl ProtocolExt for TcpOpts {
    type Opts = TcpOpts;
    type State = (); // Not used for TCP

    #[inline(always)]
    fn get_hdr_len(&self) -> usize {
        LEN_TCP_HDR
    }

    #[inline(always)]
    fn get_proto_num(&self) -> u8 {
        6
    }

    #[inline(always)]
    fn get_src_port(&self) -> Option<u16> {
        self.src_port
    }

    #[inline(always)]
    fn gen_src_port(&self, buff: &mut [u8], seed: &mut u64) -> Result<(Option<u16>, bool)> {
        // Initialize TCP header so we can set the port.
        let mut tcph = match MutableTcpPacket::new(buff) {
            Some(p) => p,
            None => {
                return Err(anyhow!(
                    "Failed to create mutable TCP packet for source port generation"
                ));
            }
        };

        let (port, is_static) = match self.src_port {
            Some(port) => (Some(port), true),
            None => (Some(rand_num(seed, 1, u16::MAX as u64) as u16), false),
        };

        // Set the source port in the TCP header
        if let Some(port) = port {
            tcph.set_source(port);
        }

        Ok((port, is_static))
    }

    #[inline(always)]
    fn get_dst_port(&self) -> Option<u16> {
        self.dst_port
    }

    #[inline(always)]
    fn gen_dst_port(&self, buff: &mut [u8], seed: &mut u64) -> Result<(Option<u16>, bool)> {
        // Initialize TCP header so we can set the port.
        let mut tcph = match MutableTcpPacket::new(buff) {
            Some(p) => p,
            None => {
                return Err(anyhow!(
                    "Failed to create mutable TCP packet for destination port generation"
                ));
            }
        };

        let (port, is_static) = match self.dst_port {
            Some(port) => (Some(port), true),
            None => (Some(rand_num(seed, 1, u16::MAX as u64) as u16), false),
        };

        // Set the destination port in the TCP header
        if let Some(port) = port {
            tcph.set_destination(port);
        }

        Ok((port, is_static))
    }

    #[inline(always)]
    fn gen_checksum(&self, buff: &mut [u8]) -> Result<()> {
        // We need to retrieve the IP header fields to calculate the checksum, so we can get the source and destination IPs from there.
        let iph = match MutableIpv4Packet::new(buff[..IP_HDR_LEN].as_mut()) {
            Some(p) => p,
            None => {
                return Err(anyhow!(
                    "Failed to create mutable IPv4 packet for TCP checksum calculation"
                ));
            }
        };

        let src_ip = iph.get_source();
        let dst_ip = iph.get_destination();

        let mut tcph = match MutableTcpPacket::new(buff[IP_HDR_LEN..].as_mut()) {
            Some(p) => p,
            None => {
                return Err(anyhow!(
                    "Failed to create mutable TCP packet for checksum calculation"
                ));
            }
        };

        tcph.set_checksum(
            ipv4_checksum(&tcph.to_immutable(), &src_ip, &dst_ip)
                .try_into()
                .map_err(|e| anyhow!("Failed to convert checksum to u16: {}", e))?,
        );

        Ok(())
    }

    #[inline(always)]
    fn fill_init(&self, buff: &mut [u8], seed: &mut u64) -> Result<(bool, bool)> {
        {
            let mut tcph = match MutableTcpPacket::new(buff) {
                Some(p) => p,
                None => {
                    return Err(anyhow!(
                        "Failed to create mutable TCP packet for filling fields"
                    ));
                }
            };

            // Fill out general fields.
            tcph.set_data_offset(5);

            // Set flags
            tcph.set_flags(self.flags);
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
        // Regenerate ports if they were not static.
        if (flags & FILL_FLAG_SRC_PORT != 0) && self.src_port.is_none() {
            self.gen_src_port(buff, seed)
                .map_err(|e| anyhow!("Failed to generate source port: {}", e))?;
        }

        if (flags & FILL_FLAG_DST_PORT != 0) && self.dst_port.is_none() {
            self.gen_dst_port(buff, seed)
                .map_err(|e| anyhow!("Failed to generate destination port: {}", e))?;
        }

        Ok(())
    }
}
