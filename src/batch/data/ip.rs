pub mod source;

use std::{net::IpAddr, str::FromStr};

use anyhow::{Result, anyhow};

use crate::{
    config::batch::data::ip::IpOpts as IpOptsCfg,
    util::{
        get_src_ip_from_ifname,
        net::{NetIpType, parse_ip_or_cidr},
    },
};

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
            src: None,
            dst: None,
            tos: None,
            ttl_min: None,
            ttl_max: None,
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
}
