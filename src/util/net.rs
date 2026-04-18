use std::net::{IpAddr, Ipv4Addr};
use std::str::FromStr;

use std::process::Command;

use anyhow::{Result, anyhow};

use std::fs;

/// Chooses a random IP from a specific CIDR range (decimal format).
///
/// # Arguments
/// * `net` - The range net IP.
/// * `cidr` - The CIDR prefix length (e.g., 24 for a /24).
/// * `seed` - Seed value used for random number generation.
///
/// # Returns
/// A random IP address as a string within the specified CIDR range.
pub fn get_rand_ip_from_cidr(net: IpAddr, cidr: u8, seed: u32) -> Result<IpAddr> {
    let net_u32 = match net {
        IpAddr::V4(v4) => u32::from(v4),
        IpAddr::V6(_) => return Err(anyhow!("IPv6 not supported at this time")),
    };

    // Build the host mask: the complement of the network mask.
    // e.g. /24  =>  mask = 0x000000FF
    let mask: u32 = if cidr == 0 {
        u32::MAX // 0 means the entire address space is the host part
    } else {
        (1u32 << (32 - cidr)).wrapping_sub(1)
    };

    // Cheap LCG that mimics the behaviour of a seeded rand_r call.
    let rand_num = lcg_rand(seed);

    // Combine the network prefix with the random host part.
    let rand_ip_u32 = (net_u32 & !mask) | (mask & rand_num);

    Ok(IpAddr::from(rand_ip_u32))
}

/// Chooses a random IP from a specific CIDR range.
///
/// # Arguments
/// * `range` - The range in "IP/CIDR" format (e.g., "192.168.1.0/24")
/// * `seed`  - Seed value used for random number generation
///
/// # Returns
/// A random IP address as a string within the specified CIDR range.
pub fn get_rand_ip_from_str(range: &str, seed: u32) -> Result<String> {
    // Split net IP/CIDR from the range string.
    let (s_ip, cidr_str) = range
        .split_once('/')
        .ok_or_else(|| anyhow!("Invalid range format, expected 'IP/CIDR'"))?;

    // Parse the CIDR prefix length.
    let cidr: u8 = match cidr_str.parse() {
        Ok(v) if v <= 32 => v,
        _ => return Err(anyhow::anyhow!("Invalid CIDR value")),
    };

    // Parsed the net IP and generate a random IP from the CIDR range.
    let net_ip = match IpAddr::from_str(s_ip) {
        Ok(ip) => ip,
        Err(e) => return Err(anyhow!("Invalid IP address '{}': {}", s_ip, e)),
    };

    get_rand_ip_from_cidr(net_ip, cidr, seed).map(|ip| ip.to_string())
}

/// Internal function that acts as a minimal LCG random number generator that mirrors the single-step
/// output of a typical `rand_r` implementation (glibc / POSIX).
///
/// glibc rand_r does:  next = next * 1103515245 + 12345
/// and returns        (next >> 16) & 0x7FFF  (a 15-bit value)
///
/// We return the full 32-bit scrambled seed so the host bits are filled
/// across the whole mask (important for prefix lengths smaller than 17).
fn lcg_rand(seed: u32) -> u32 {
    seed.wrapping_mul(1_103_515_245).wrapping_add(12_345)
}

/// Retrieves the source MAC address of a network interface.
///
/// Reads from `/sys/class/net/<dev>/address`, which on Linux exposes
/// the interface's MAC as a human-readable `"aa:bb:cc:dd:ee:ff\n"` string.
///
/// # Arguments
/// * `dev` - The interface/device name (e.g. `"eth0"`, `"ens3"`)
///
/// # Returns
/// `Ok([u8; 6])` containing the 6 MAC address bytes on success,
/// or an `Err` describing what went wrong.
pub fn get_src_mac_addr(dev: &str) -> Result<[u8; 6]> {
    let path = format!("/sys/class/net/{}/address", dev);

    // Read and trim the file in one shot.
    let contents =
        fs::read_to_string(&path).map_err(|e| anyhow!("failed to open '{}': {}", path, e))?;

    let mac_str = contents.trim();

    // Parse "aa:bb:cc:dd:ee:ff" into 6 bytes.
    let bytes: Vec<u8> = mac_str
        .split(':')
        .map(|octet| u8::from_str_radix(octet, 16))
        .collect::<Result<_, _>>()
        .map_err(|e| anyhow!("failed to parse MAC address '{}': {}", mac_str, e))?;

    // Ensure we got exactly 6 bytes.
    let mac = <[u8; 6]>::try_from(bytes.as_slice())
        .map_err(|_| anyhow!("expected 6 MAC octets, received {}", bytes.len()))?;

    Ok(mac)
}

/// Retrieves the Ethernet MAC address of the host's default gateway.
///
/// # Notes
/// Shells out to `ip` to find the default gateway, then looks it up
/// in the table to get its MAC address.
/// e.g., `ip -4 route list 0/0 | cut -d' ' -f3`
///
/// # Returns
/// `Ok([u8; 6])` containing the 6 MAC address bytes on success,
/// or an `Err` describing what went wrong.
pub fn get_gw_mac() -> Result<[u8; 6]> {
    // Get the default gateway IP: `ip -4 route list 0/0 | cut -d' ' -f3`
    let gw_ip = Command::new("ip")
        .args(["-4", "route", "list", "0/0"])
        .output()
        .map_err(|e| anyhow!("failed to run 'ip route': {}", e))?;

    let gw_ip = String::from_utf8_lossy(&gw_ip.stdout);
    let gw_ip = gw_ip
        .split_whitespace()
        .nth(2)
        .ok_or_else(|| anyhow!("failed to parse default gateway from 'ip route' output"))?;

    // Look up the gateway in the neighbour table.
    // NOTE: Try `ip neigh show <gw_ip>`
    let neigh = Command::new("ip")
        .args(["neigh", "show", gw_ip])
        .output()
        .map_err(|e| anyhow!("failed to run 'ip neigh {}': {}", gw_ip, e))?;

    let neigh = String::from_utf8_lossy(&neigh.stdout);
    let mac_str = neigh
        .split_whitespace()
        .find(|s| s.chars().filter(|c| *c == ':').count() == 5)
        .ok_or_else(|| {
            anyhow!(
                "failed to find MAC address in 'ip neigh' output for '{}'",
                gw_ip
            )
        })?;

    // Parse MAC address into bytes.
    let bytes: Vec<u8> = mac_str
        .split(':')
        .map(|octet| u8::from_str_radix(octet, 16))
        .collect::<Result<_, _>>()
        .map_err(|e| anyhow!("failed to parse MAC address '{}': {}", mac_str, e))?;

    <[u8; 6]>::try_from(bytes.as_slice())
        .map_err(|_| anyhow!("expected 6 MAC octets, received {}", bytes.len()))
}

/// Attempts to determine the network interface name associated with a given source IP address.
///
/// # Arguments
/// * `src_ip` - The source IP address to look up (e.g., "192.168.1.1")
///
/// # Returns
/// `Ok(String)` containing the interface name (e.g., "eth0") on success,
/// or an `Err` describing what went wrong.
pub fn get_ifname_from_src_ip(src_ip: &str) -> Result<String> {
    let output = Command::new("ip")
        .args(["-o", "-4", "addr", "show", "to", src_ip])
        .output()
        .map_err(|e| anyhow!("failed to run 'ip addr show': {}", e))?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    let ifname = output_str
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| anyhow!("failed to parse interface name from 'ip addr' output"))?;

    Ok(ifname.to_string())
}

/// Attempts to retrieve the source IP of the specified interface.
///
/// # Arguments
/// * `if_name` - The name of the interface to query (e.g., "eth0")
///
/// # Returns
/// `Ok(String)` containing the source IP address (e.g., "192.168.1.1") on success,
/// or an `Err` describing what went wrong.
pub fn get_src_ip_from_ifname(if_name: &str) -> Result<String> {
    let output = Command::new("ip")
        .args(["-o", "-4", "addr", "show", "dev", if_name])
        .output()
        .map_err(|e| anyhow!("failed to run 'ip addr show': {}", e))?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    let src_ip = output_str
        .split_whitespace()
        .find(|s| s.contains('/'))
        .and_then(|s| s.split('/').next())
        .ok_or_else(|| anyhow!("failed to parse source IP from 'ip addr' output"))?;

    Ok(src_ip.to_string())
}

/// Parses a MAC address string in the format "aa:bb:cc:dd:ee:ff" into a 6-byte array.
///
/// # Arguments
/// * `mac_str` - The MAC address string to parse (e.g., "00:1A:2B:3C:4D:5E")
///
/// # Returns
/// `Ok([u8; 6])` containing the 6 MAC address bytes on success,
/// or an `Err` describing what went wrong.
pub fn get_mac_addr_from_str(mac_str: &str) -> Result<[u8; 6]> {
    let bytes: Vec<u8> = mac_str
        .split(':')
        .map(|octet| u8::from_str_radix(octet, 16))
        .collect::<Result<_, _>>()
        .map_err(|e| anyhow!("failed to parse MAC address '{}': {}", mac_str, e))?;

    <[u8; 6]>::try_from(bytes.as_slice())
        .map_err(|_| anyhow!("expected 6 MAC octets, received {}", bytes.len()))
}

pub struct NetIpMulti {
    pub net: String,
    pub cidr: u8,
}
pub enum NetIpType {
    Single(String),
    Multi(NetIpMulti),
}

/// Checks if the given string contains a valid IP address. If the string is a network IP with a CIDR, it will return a structure containing the net IP and CIDR. Otherwise, it will return the string as a single IP.
pub fn parse_ip_or_cidr(ip_str: &str) -> Result<NetIpType> {
    if ip_str.contains('/') {
        let mut parts = ip_str.splitn(2, '/');

        match (parts.next(), parts.next()) {
            (Some(net), Some(cidr)) => {
                let cidr = match cidr.parse() {
                    Ok(v) if v <= 32 => v,
                    _ => return Err(anyhow!("Invalid CIDR value: {}", cidr)),
                };

                // If we have a /32 CIDR, treat as a single IP.
                if cidr == 32 {
                    return Ok(NetIpType::Single(net.to_string()));
                } else {
                    return Ok(NetIpType::Multi(NetIpMulti {
                        net: net.to_string(),
                        cidr,
                    }));
                }
            }
            _ => return Err(anyhow::anyhow!("Invalid CIDR format")),
        }
    }

    // Validate that it's a valid IP address.
    ip_str
        .parse::<Ipv4Addr>()
        .map_err(|e| anyhow!("Invalid IP address '{}': {}", ip_str, e))
        .map(|_| NetIpType::Single(ip_str.to_string()))
}
