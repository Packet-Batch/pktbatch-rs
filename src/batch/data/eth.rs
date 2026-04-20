use anyhow::{Result, anyhow};
use pnet::packet::ethernet::{EtherTypes, MutableEthernetPacket};

use crate::{
    config::batch::data::eth::EthOpts as EthOptsCfg,
    util::{get_gw_mac, get_mac_addr_from_str, get_src_mac_addr},
};

pub const ETH_HDR_LEN: usize = 14;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct EthOpts {
    pub src_mac: Option<String>,
    pub dst_mac: Option<String>,
}

impl From<EthOptsCfg> for EthOpts {
    fn from(cfg: EthOptsCfg) -> Self {
        Self {
            src_mac: cfg.src_mac,
            dst_mac: cfg.dst_mac,
        }
    }
}

impl EthOpts {
    /// Attemps to retrieve the source MAC address. If `src_mac` is specified, it will be parsed and returned. If not, the function will attempt to retrieve the gateway MAC address. Errors during parsing or retrieval will be returned as `anyhow::Error`.
    ///
    /// # Returns
    /// A `Result` containing the source MAC address as a 6-byte array if successful, or an `anyhow::Error` if parsing or retrieval fails.
    pub fn get_src_mac(&self, if_name: Option<String>) -> Result<[u8; 6]> {
        match &self.src_mac {
            Some(mac_str) => get_mac_addr_from_str(&mac_str)
                .map_err(|e| anyhow!("Failed to parse source MAC address {}: {}", mac_str, e)),
            None => match if_name {
                Some(name) => get_src_mac_addr(&name),
                None => Err(anyhow!(
                    "Interface name not provided for source MAC retrieval"
                )),
            },
        }
    }

    /// Attemps to retrieve the destination MAC address. If `dst_mac` is specified, it will be parsed and returned. If not, the function will attempt to retrieve the gateway MAC address. Errors during parsing or retrieval will be returned as `anyhow::Error`.
    ///
    /// # Returns
    /// A `Result` containing the destination MAC address as a 6-byte array if successful, or an `anyhow::Error` if parsing or retrieval fails.
    pub fn get_dst_mac(&self) -> Result<[u8; 6]> {
        match &self.dst_mac {
            Some(mac_str) => get_mac_addr_from_str(&mac_str)
                .map_err(|e| anyhow!("Failed to parse destination MAC address {}: {}", mac_str, e)),
            None => match get_gw_mac() {
                Ok(mac) => Ok(mac),
                Err(e) => Err(anyhow!(
                    "Failed to get gateway MAC 
                address: {}",
                    e
                )),
            },
        }
    }

    /// Fills the ethernet header fields initially.
    ///
    /// # Arguments
    /// * `buff` - The buffer to write the ethernet header into.
    ///
    /// # Returns
    /// A `Result` indicating success or failure of the header filling operation. Errors during MAC address retrieval or buffer manipulation will be returned as `anyhow::Error`.
    pub fn fill_init(&self, buff: &mut [u8], if_name: Option<String>) -> Result<()> {
        let src_mac = self.get_src_mac(if_name)?;
        let dst_mac = self.get_dst_mac()?;

        // Write MAC addresses into the buffer.
        {
            let mut eth = match MutableEthernetPacket::new(buff) {
                Some(p) => p,
                None => {
                    return Err(anyhow!(
                        "Failed to create mutable Ethernet packet for header filling"
                    ));
                }
            };

            eth.set_source(src_mac.into());
            eth.set_destination(dst_mac.into());

            eth.set_ethertype(EtherTypes::Ipv4);
        }

        Ok(())
    }
}
