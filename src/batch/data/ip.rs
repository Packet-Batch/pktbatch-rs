use std::{net::IpAddr, str::FromStr};

use anyhow::{Result, anyhow};
use pnet::packet::{
    ip::IpNextHeaderProtocols,
    ipv4::{MutableIpv4Packet, checksum},
};

use crate::{
    batch::data::protocol::Protocol,
    config::batch::data::ip::IpOpts as IpOptsCfg,
    util::{
        get_rand_ip_from_cidr, get_src_ip_from_ifname,
        net::{NetIpType, parse_ip_or_cidr},
        rand_num,
    },
};

// 5 x 32-bit words = 20 bytes (options not supported right now)
pub const IP_IHL: usize = 5;

pub const IP_HDR_LEN: usize = (IP_IHL * 4) as usize;

pub const DEF_IP_TTL: u8 = 64;
pub const DEF_IP_TOS: u8 = 0x08; // Default to "low delay" as per RFC 791.

pub const FILL_FLAG_IP_SRC: u32 = 1 << 0;
pub const FILL_FLAG_IP_DST: u32 = 1 << 1;
pub const FILL_FLAG_IP_ID: u32 = 1 << 2;
pub const FILL_FLAG_IP_TTL: u32 = 1 << 3;

#[derive(Debug, Clone)]
pub struct FullIpAddr {
    pub ip: IpAddr,
    pub cidr: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IpOpts {
    pub src: Option<Vec<String>>,
    pub dst: Option<Vec<String>>,

    pub tos: Option<u8>,

    pub ttl_min: Option<u8>,
    pub ttl_max: Option<u8>,

    pub id_min: Option<u16>,
    pub id_max: Option<u16>,

    pub do_csum: bool,
}

impl Default for IpOpts {
    fn default() -> Self {
        IpOpts {
            src: Some(vec!["127.0.0.1".to_string()]), // Will try loopback interface by default?
            dst: None,
            tos: Some(DEF_IP_TOS),
            ttl_min: Some(DEF_IP_TTL),
            ttl_max: Some(DEF_IP_TTL),
            id_min: None,
            id_max: None,
            do_csum: true,
        }
    }
}

impl From<IpOptsCfg> for IpOpts {
    fn from(cfg: IpOptsCfg) -> Self {
        Self {
            src: cfg.srcs.or_else(|| cfg.src.map(|s| vec![s])),
            dst: cfg.dsts.or_else(|| cfg.dst.map(|s| vec![s])),
            tos: cfg.tos,
            ttl_min: cfg.ttl_min,
            ttl_max: cfg.ttl_max,
            id_min: cfg.id_min,
            id_max: cfg.id_max,
            do_csum: cfg.do_csum,
        }
    }
}

impl IpOpts {
    /// Retrieves the source IP addresses based on the configuration. If `src` is specified, it will parse each entry as either a single IP or CIDR notation and return a vector of `FullIpAddr`. If `src` is not specified, it will attempt to retrieve the source IP from the provided interface name. Errors during parsing or retrieval will be returned as `anyhow::Error`.
    ///
    /// # Arguments
    /// * `if_name` - An optional interface name to retrieve the source IP from if `src` is not specified.
    ///
    /// # Returns
    /// A `Result` containing a vector of `FullIpAddr` if successful, or an `anyhow::Error` if parsing or retrieval fails.
    pub fn get_src_ips(&self, if_name: Option<&str>) -> Result<Vec<FullIpAddr>> {
        let ips = match &self.src {
            Some(src) => src
                .iter()
                .filter_map(|ip_str| {
                    parse_ip_or_cidr(ip_str)
                        .and_then(|t| match t {
                            NetIpType::Single(ip) => IpAddr::from_str(&ip)
                                .map(|ip| FullIpAddr { ip, cidr: 32 })
                                .map_err(|e| anyhow!("failed to parse source IP {}: {}", ip, e)),
                            NetIpType::Multi(t) => IpAddr::from_str(&t.net)
                                .map(|ip| FullIpAddr { ip, cidr: t.cidr })
                                .map_err(|e| anyhow!("failed to parse source IP {}: {}", t.net, e)),
                        })
                        .ok()
                })
                .collect(),
            None => {
                let Some(if_name) = if_name else {
                    return Err(anyhow!(
                        "no source IPs specified and no interface name provided"
                    ));
                };

                let src_ip = get_src_ip_from_ifname(if_name).map_err(|e| {
                    anyhow!("failed to get source IP from interface {}: {}", if_name, e)
                })?;

                let ip = src_ip
                    .parse::<IpAddr>()
                    .map_err(|e| anyhow!("failed to parse source IP {}: {}", src_ip, e))?;

                vec![FullIpAddr { ip, cidr: 32 }]
            }
        };

        Ok(ips)
    }

    /// Generates the initial or next source IP address.
    ///
    /// # Arguments
    /// * `ips` - A slice of `FullIpAddr` representing the available source IP addresses or CIDR ranges to choose from.
    /// * `seed` - A mutable reference to a `u64` seed value used for random number generation when selecting an IP address from a CIDR range or from a list of IPs.
    ///
    /// # Returns
    /// A `Result` containing the generated `IpAddr` if successful along with whether it's static, or an `anyhow::Error` if there was an issue generating the IP address (e.g., invalid CIDR range, no IPs available to choose from, etc.).
    #[inline(always)]
    pub fn get_next_src_ip(&self, ips: &[FullIpAddr], seed: &mut u64) -> Result<(IpAddr, bool)> {
        let idx = if ips.len() <= 1 {
            0
        } else {
            // Use your fast math-based range function
            rand_num(seed, 0, (ips.len() - 1) as u64) as usize
        };

        let ip_full = match ips.get(idx) {
            Some(net) => net,
            None => return Err(anyhow!("No source IPs available to choose from")),
        };

        // If this is a /32, just return.
        if ip_full.cidr == 32 {
            return Ok((ip_full.ip, ips.len() == 1));
        }

        // Otherwise, generate a random IP from the CIDR range.
        get_rand_ip_from_cidr(ip_full.ip, ip_full.cidr, seed)
            .map_err(|e| {
                anyhow!(
                    "Failed to generate random source IP from CIDR {}/{}: {}",
                    ip_full.ip,
                    ip_full.cidr,
                    e
                )
            })
            .map(|ip| (ip, false))
    }

    /// Generates the source IP address and fills out the buffer.
    ///
    /// # Arguments
    /// * `buff` - The buffer to write the generated source IP into, starting at the correct offset for the IP header.
    /// * `ips` - A slice of `FullIpAddr` representing the available source IP addresses or CIDR ranges to choose from when generating the source IP.
    /// * `seed` - A mutable reference to a `u64` seed value used for random number generation when selecting an IP address from a CIDR range or from a list of IPs.
    ///
    /// # Returns
    /// A `Result` indicating success or failure of the source IP generation and buffer filling operation along with whether it's a static source IP. Errors may occur if there is an issue generating the source IP address (e.g., invalid CIDR range, no IPs available to choose from, etc.) or if there is an issue with creating the mutable IP packet from the buffer.
    #[inline(always)]
    pub fn gen_src_ip(&self, buff: &mut [u8], ips: &[FullIpAddr], seed: &mut u64) -> Result<bool> {
        let (src_ip, is_static) = self.get_next_src_ip(ips, seed)?;

        let mut iph = match MutableIpv4Packet::new(buff) {
            Some(pkt) => pkt,
            None => return Err(anyhow!("Failed to create mutable IPv4 packet from buffer")),
        };

        iph.set_source(match src_ip {
            IpAddr::V4(ipv4) => ipv4.into(),
            IpAddr::V6(_) => {
                return Err(anyhow!("IPv6 addresses are not supported for source IP"));
            }
        });

        Ok(is_static)
    }

    /// Retrieves the destination IP addresses based on the configuration. If `dst` is specified, it will parse each entry as either a single IP or CIDR notation and return a vector of `FullIpAddr`. If `dst` is not specified, an error will be returned indicating that at least one destination IP must be specified for packet generation. Errors during parsing will also be returned as `anyhow::Error`.
    ///
    /// # Returns
    /// A `Result` containing a vector of `FullIpAddr` if successful, or an `anyhow::Error` if parsing fails or if no destination IPs are specified.
    pub fn get_dst_ips(&self) -> Result<Vec<FullIpAddr>> {
        let ips = match &self.dst {
            Some(dst) => dst
                .iter()
                .filter_map(|ip_str| {
                    parse_ip_or_cidr(ip_str)
                        .and_then(|t| match t {
                            NetIpType::Single(ip) => IpAddr::from_str(&ip)
                                .map(|ip| FullIpAddr { ip, cidr: 32 })
                                .map_err(|e| {
                                    anyhow!("failed to parse destination IP {}: {}", ip, e)
                                }),
                            NetIpType::Multi(t) => IpAddr::from_str(&t.net)
                                .map(|ip| FullIpAddr { ip, cidr: t.cidr })
                                .map_err(|e| {
                                    anyhow!("failed to parse destination IP {}: {}", t.net, e)
                                }),
                        })
                        .ok()
                })
                .collect(),
            None => {
                return Err(anyhow!(
                    "No destination IPs specified. At least one destination IP must be specified for packet generation."
                ));
            }
        };

        Ok(ips)
    }

    /// Generates the initial or next destination IP address.
    ///
    /// # Arguments
    /// * `ips` - A slice of `FullIpAddr` representing the available destination IP addresses or CIDR ranges to choose from.
    /// * `seed` - A mutable reference to a `u64` seed value used for random number generation when selecting an IP address from a CIDR range or from a list of IPs.
    ///
    /// # Returns
    /// A `Result` containing the next `IpAddr` if successful along with whether it's static, or an `anyhow::Error` if there was an issue generating the IP address (e.g., invalid CIDR range, no IPs available to choose from, etc.).
    #[inline(always)]
    pub fn get_next_dst_ip(&self, ips: &[FullIpAddr], seed: &mut u64) -> Result<(IpAddr, bool)> {
        let idx = if ips.len() <= 1 {
            0
        } else {
            // Use your fast math-based range function
            rand_num(seed, 0, (ips.len() - 1) as u64) as usize
        };

        let ip_full = match ips.get(idx) {
            Some(net) => net,
            None => return Err(anyhow!("No destination IPs available to choose from")),
        };

        // If this is a /32, just return.
        if ip_full.cidr == 32 {
            return Ok((ip_full.ip, ips.len() == 1));
        }

        // Otherwise, generate a random IP from the CIDR range.
        get_rand_ip_from_cidr(ip_full.ip, ip_full.cidr, seed)
            .map_err(|e| {
                anyhow!(
                    "Failed to generate random destination IP from CIDR {}/{}: {}",
                    ip_full.ip,
                    ip_full.cidr,
                    e
                )
            })
            .map(|ip| (ip, false))
    }

    /// Generates the destination IP address and fills out the buffer.
    ///
    /// # Arguments
    /// * `buff` - The buffer to write the generated destination IP into, starting at the correct offset for the IP header.
    /// * `ips` - A slice of `FullIpAddr` representing the available destination IP addresses or CIDR ranges to choose from when generating the destination IP.
    /// * `seed` - A mutable reference to a `u64` seed value used for random number generation when selecting an IP address from a CIDR range or from a list of IPs.
    ///
    /// # Returns
    /// A `Result` indicating success or failure of the destination IP generation and buffer filling operation. Errors may occur if there is an issue generating the destination IP address (e.g., invalid CIDR range, no IPs available to choose from, etc.) or if there is an issue with creating the mutable IP packet from the buffer.
    #[inline(always)]
    pub fn gen_dst_ip(&self, buff: &mut [u8], ips: &[FullIpAddr], seed: &mut u64) -> Result<bool> {
        let (dst_ip, is_static) = self.get_next_dst_ip(ips, seed)?;

        let mut iph = match MutableIpv4Packet::new(buff) {
            Some(pkt) => pkt,
            None => return Err(anyhow!("Failed to create mutable IPv4 packet from buffer")),
        };

        iph.set_destination(match dst_ip {
            IpAddr::V4(ipv4) => ipv4.into(),
            IpAddr::V6(_) => {
                return Err(anyhow!(
                    "IPv6 addresses are not supported for destination IP"
                ));
            }
        });

        Ok(is_static)
    }

    /// Fills the IP header fields initially. This is the initial filling before the thread packet generation loop. Used for static fields.
    ///
    /// # Arguments
    /// * `buff` - The packet buffer starting at the IP header.
    /// * `seed` - A mutable reference to a seed value for generating random fields if needed.
    /// * `proto` - The protocol for the packet, used to set the correct protocol field in the IP header.
    /// * `src_ips` - A slice of `FullIpAddr` representing the available source IP addresses or CIDR ranges to choose from when generating the source IP.
    /// * `dst_ips` - A slice of `FullIpAddr` representing the available destination IP addresses or CIDR ranges to choose from when generating the destination IP.
    ///
    /// # Returns
    /// A `Result` indicating success or failure of filling the IP header fields along with whether the source and destination IPs are static along with the ID and TTL fields. Errors may occur if the packet buffer is not properly formatted or if there is an issue with generating random values for the fields.
    #[inline(always)]
    pub fn fill_init(
        &self,
        buff: &mut [u8],
        seed: &mut u64,
        proto: &Protocol,
        src_ips: &[FullIpAddr],
        dst_ips: &[FullIpAddr],
    ) -> Result<(bool, bool, bool, bool)> {
        let buff_sz = buff.len();

        // Construct the IPv4 header now and fill it out.
        let mut iph = match MutableIpv4Packet::new(buff) {
            Some(pkt) => pkt,
            None => return Err(anyhow!("Failed to create mutable IPv4 packet from buffer")),
        };

        iph.set_version(4);
        iph.set_header_length(IP_IHL as u8);
        iph.set_total_length(buff_sz as u16);

        // Set protocol field based on batch config.
        iph.set_next_level_protocol(match proto {
            Protocol::Tcp(_) => IpNextHeaderProtocols::Tcp,
            Protocol::Udp(_) => IpNextHeaderProtocols::Udp,
            Protocol::Icmp(_) => IpNextHeaderProtocols::Icmp,
        });

        let static_ttl = self.ttl_min.and_then(|min| {
            self.ttl_max
                .and_then(|max| if min == max { Some(min) } else { None })
        });

        // Set TTL based off of batch config.
        iph.set_ttl(static_ttl.unwrap_or_else(|| {
            rand_num(
                seed,
                self.ttl_min.unwrap_or(DEF_IP_TTL) as u64,
                self.ttl_max.unwrap_or(DEF_IP_TTL) as u64,
            ) as u8
        }));

        let static_id = self.id_min.and_then(|min| {
            self.id_max
                .and_then(|max| if min == max { Some(min) } else { None })
        });

        // Set ID field based on random/static configuration.
        iph.set_identification(static_id.unwrap_or_else(|| {
            rand_num(
                seed,
                self.id_min.unwrap_or(0) as u64,
                self.id_max.unwrap_or(u16::MAX) as u64,
            ) as u16
        }));

        // We don't support fragmentation.
        iph.set_fragment_offset(0);

        // Set ToS field. We default to 0x08 (low delay) if not specified as per RFC 791.
        let tos = self.tos.unwrap_or(DEF_IP_TOS);

        iph.set_dscp(tos >> 2);
        iph.set_ecn(tos & 0x03);

        // Generate source and destination IPs and set them in the header.
        let is_static_src = self
            .gen_src_ip(buff, src_ips, seed)
            .map_err(|e| anyhow!("Failed to generate source IP: {}", e))?;

        let is_static_dst = self
            .gen_dst_ip(buff, dst_ips, seed)
            .map_err(|e| anyhow!("Failed to generate destination IP: {}", e))?;

        Ok((
            is_static_src,
            is_static_dst,
            static_id.is_some(),
            static_ttl.is_some(),
        ))
    }

    /// Fills the IP header fields for the next packet. This is called inside the thread packet generation loop after the initial filling. Used for fields that may need to be regenerated for each packet (e.g., if they are not static).
    ///
    /// # Arguments
    /// * `buff` - The packet buffer starting at the IP header.
    /// * `flags` - A bitmask of flags indicating which fields need to be refilled. This allows the function to know which fields to regenerate based on whether they were static or not during the initial filling.
    /// * `seed` - A mutable reference to a seed value for generating random fields if needed.
    /// * `src_ips` - A slice of `FullIpAddr` representing the available source IP addresses or CIDR ranges to choose from when generating the source IP if it needs to be refilled.
    /// * `dst_ips` - A slice of `FullIpAddr` representing the available destination IP addresses or CIDR ranges to choose from when generating the destination IP if it needs to be refilled.
    ///
    /// # Returns
    /// A `Result` indicating success or failure of filling the IP header fields for the next packet. Errors may occur if the packet buffer is not properly formatted or if there is an issue with generating random values for the fields that need to be refilled.
    pub fn fill(
        &self,
        buff: &mut [u8],
        flags: u32,
        seed: &mut u64,
        src_ips: &[FullIpAddr],
        dst_ips: &[FullIpAddr],
    ) -> Result<()> {
        {
            // Regenerate source IP if it was not static.
            if flags & FILL_FLAG_IP_SRC != 0 {
                self.gen_src_ip(buff, src_ips, seed)
                    .map_err(|e| anyhow!("Failed to generate source IP: {}", e))?;
            }

            // Regenerate destination IP if it was not static.
            if flags & FILL_FLAG_IP_DST != 0 {
                self.gen_dst_ip(buff, dst_ips, seed)
                    .map_err(|e| anyhow!("Failed to generate destination IP: {}", e))?;
            }
        }

        {
            let mut iph = match MutableIpv4Packet::new(buff) {
                Some(pkt) => pkt,
                None => return Err(anyhow!("Failed to create mutable IPv4 packet from buffer")),
            };

            // Regenerate ID if it was not static.
            if flags & FILL_FLAG_IP_ID != 0 {
                iph.set_identification(rand_num(
                    seed,
                    self.id_min.unwrap_or(0) as u64,
                    self.id_max.unwrap_or(u16::MAX) as u64,
                ) as u16);
            }

            // Regenerate TTL if it was not static.
            if flags & FILL_FLAG_IP_TTL != 0 {
                let ttl = rand_num(
                    seed,
                    self.ttl_min.unwrap_or(DEF_IP_TTL) as u64,
                    self.ttl_max.unwrap_or(DEF_IP_TTL) as u64,
                ) as u8;

                iph.set_ttl(ttl);
            }
        }

        Ok(())
    }

    /// Generates the checksum for the IP header and sets it in the header. This should be called after all other fields have been filled out and before the packet is sent.
    ///
    /// # Arguments
    /// * `buff` - The packet buffer starting at the IP header, which should already have all the fields filled out that are needed for checksum calculation.
    ///
    /// # Returns
    /// A `Result` indicating success or failure of the checksum generation and setting operation. Errors may occur if the packet buffer is not properly formatted or if there is an issue with creating the mutable IP packet from the buffer.
    #[inline(always)]
    pub fn gen_checksum(&self, buff: &mut [u8]) -> Result<()> {
        if !self.do_csum {
            return Ok(());
        }

        let mut iph = match MutableIpv4Packet::new(buff) {
            Some(pkt) => pkt,
            None => return Err(anyhow!("Failed to create mutable IPv4 packet from buffer")),
        };

        iph.set_checksum(checksum(&iph.to_immutable()));

        Ok(())
    }
}
