use std::{net::IpAddr, str::FromStr, thread};

use anyhow::{Result, anyhow};
use pnet::{
    datalink::EtherType,
    packet::{
        MutablePacket,
        ethernet::{EtherTypes, MutableEthernetPacket},
        ip::IpNextHeaderProtocols,
        ipv4::MutableIpv4Packet,
    },
};

use crate::{
    batch::data::{BatchData, ip::FullIpAddr, protocol::Protocol},
    context::Context,
    logger::level::LogLevel,
    util::{
        get_cpu_rdtsc, get_gw_mac, get_mac_addr_from_str, get_src_ip_from_ifname,
        net::{NetIpType, get_src_mac_addr},
        rand_num,
        sys::get_cpu_count,
    },
};

use rand::{SeedableRng, seq::SliceRandom};

use rand::rngs::StdRng;

const MAX_BUFFER_SZ: usize = 2048;

const DEF_IP_TTL: u8 = 64;
const DEF_IP_TOS: u8 = 0x08; // Default to "low delay" as per RFC 791.

impl BatchData {
    pub fn exec(&self, ctx: Context, id: u16) -> Result<()> {
        // Retrieve the number of threads we should create.
        let thread_cnt = if self.thread_cnt > 0 {
            self.thread_cnt
        } else {
            get_cpu_count() as u16
        };

        // Prepare block handles.
        let mut block_hdl = Vec::new();

        // Spawn threads.
        for i in 0..thread_cnt {
            let ctx = ctx.clone();
            let data = self.clone();

            let cfg = &ctx.cfg;

            let hdl = thread::spawn(move || {
                // We'll want to clone immutable data here so that we aren't waiting for locks from shared threads (hurts performance).
                let tech = ctx.tech.blocking_read().clone();
                let logger = ctx.logger.blocking_read().clone();
                let batch = self.clone();

                logger
                    .log_msg(
                        LogLevel::Info,
                        &format!(
                            "Starting batch execution (batch_id={}, thread_id={})",
                            id, i
                        ),
                    )
                    .ok();

                // We need to retrieve the interface name.
                let if_name = if let Some(name) = tech.if_name {
                    name
                } else {
                    // We can use our util function to get the interface name from the source IP.
                    let src_ip = batch
                        .batches
                        .first()
                        .and_then(|b| b.opt_ip.src.as_ref())
                        .and_then(|src_vec| src_vec.first())
                        .ok_or_else(|| anyhow!("No source IP found to derive interface name"))?;

                    get_ifname_from_src_ip(src_ip)
                        .ok_or_else(|| anyhow!("Could not find interface for IP {}", src_ip))?
                };

                // Retrieve protocol from batch config.
                let proto: Protocol = Protocol::from(
                    batch
                        .opt_proto
                        .clone()
                        .ok_or_else(|| anyhow!("No protocol specified in batch"))?,
                )?;

                // Determine MAC addresses now.
                let src_mac = match batch.opt_eth.unwrap_or_default().get_src_mac() {
                    Ok(mac) => mac,
                    Err(e) => {
                        logger
                            .log_msg(
                                LogLevel::Error,
                                &format!("Failed to get source MAC address: {}", e),
                            )
                            .ok();

                        return;
                    }
                };

                // Retrieve destimation MAC address using our util func.
                // If `opt_eth` is `None`, MAC addresses will be None meaning it'll try retrieving from the default gateway.
                let dst_mac = match batch.opt_eth.unwrap_or_default().get_dst_mac() {
                    Ok(mac) => mac,
                    Err(e) => {
                        logger
                            .log_msg(
                                LogLevel::Error,
                                &format!("Failed to get destination MAC address: {}", e),
                            )
                            .ok();

                        return;
                    }
                };

                // Determine source and destination IP address(es) now.
                // We'll also want to determine if the IPs are static to save resources later.
                let src_ips = match batch.opt_ip.get_src_ips(Some(&if_name)) {
                    Ok(ips) => ips,
                    Err(e) => {
                        logger
                            .log_msg(
                                LogLevel::Error,
                                &format!("Failed to retrieve source IP addresses: {}", e),
                            )
                            .ok();

                        return;
                    }
                };

                let static_src_ip = if src_ips.len() == 1 {
                    let net = src_ips.first().unwrap();

                    if net.cidr == 32 { Some(net.ip) } else { None }
                } else {
                    None
                };

                let dst_ips = match self.opt_ip.get_dst_ips() {
                    Ok(ips) => ips,
                    Err(e) => {
                        logger
                            .log_msg(
                                LogLevel::Error,
                                &format!("Failed to retrieve destination IP addresses: {}", e),
                            )
                            .ok();

                        return;
                    }
                };

                let static_dst_ip = if dst_ips.len() == 1 {
                    let net = dst_ips.first().unwrap();

                    if net.cidr == 32 { Some(net.ip) } else { None }
                } else {
                    None
                };

                // We need to determine the payload.

                // Generate seed using CPU timestamp counter for better randomness across threads.
                let mut rng = StdRng::seed_from_u64(get_cpu_rdtsc() as u64);

                // Construct the packet buffer now.
                let mut buff: [u8; MAX_BUFFER_SZ] = [0; MAX_BUFFER_SZ];

                // Create ethernet header and fill out static ethernet fields now.
                let mut eth = MutableEthernetPacket::new(&mut buff)
                    .ok_or_else(|| anyhow!("Failed to create Ethernet packet from buffer"))?;

                // Set IPv4 ether type.
                eth.set_ethertype(EtherTypes::Ipv4);

                // Set source and destination MAC addresses.
                eth.set_source(src_mac.into());
                eth.set_destination(dst_mac.into());

                // Construct the IPv4 header now and fill it out.
                let mut iph = MutableIpv4Packet::new(eth.payload_mut())
                    .ok_or_else(|| anyhow!("Failed to create IPv4 packet from Ethernet buffer"))?;

                let opt_ip = &batch.opt_ip;

                iph.set_version(4);
                iph.set_header_length(5 * 4);

                // Set TTL based off of batch config.
                let ip_static_ttl =
                    batch.opt_ip.ttl_min.unwrap_or(0) == batch.opt_ip.ttl_max.unwrap_or(1);

                iph.set_ttl(rand_num(
                    opt_ip.ttl_min.unwrap_or(DEF_IP_TTL) as u16,
                    opt_ip.ttl_max.unwrap_or(DEF_IP_TTL) as u16,
                ) as u8);

                // Set protocol field based on batch config.
                iph.set_next_level_protocol(match proto {
                    Tcp(_) => IpNextHeaderProtocols::Tcp,
                    Udp(_) => IpNextHeaderProtocols::Udp,
                    Icmp(_) => IpNextHeaderProtocols::Icmp,
                });

                // We don't support fragmentation.
                iph.set_fragment_offset(0);

                // Set ID field based on random/static configuration.
                iph.set_identification(rand_num(
                    opt_ip.id_min.unwrap_or(0),
                    opt_ip.id_max.unwrap_or(u16::MAX),
                ));

                iph.set_tos(opt_ip.tos.unwrap_or(DEF_IP_TOS));

                // Now set source IP address(es) now.
                iph.set_source(match static_src_ip {
                    Some(ip) => ip.into(),
                    None => {
                        let rand_ip: FullIpAddr = src_ips
                            .choose(&mut rng)
                            .ok_or_else(|| anyhow!("Source IP list is empty"))?;

                        if rand_ip.cidr == 32 {
                            rand_ip.ip.into()
                        } else {
                            // Generate a random IP within the CIDR range.
                            get_rand_ip_from_str(&rand_ip.to_string(), rng.next_u32())?
                                .parse::<IpAddr>()
                                .map_err(|e| anyhow!("Failed to parse generated source IP: {}", e))?
                                .into()
                        }
                    }
                });

                // Now set destination IP address(es) now.
                iph.set_destination(match static_dst_ip {
                    Some(ip) => ip.into(),
                    None => {
                        let rand_ip: FullIpAddr = dst_ips
                            .choose(&mut rng)
                            .ok_or_else(|| anyhow!("Destination IP list is empty"))?;

                        if rand_ip.cidr == 32 {
                            rand_ip.ip.into()
                        } else {
                            // Generate a random IP within the CIDR range.
                            get_rand_ip_from_str(&rand_ip.to_string(), rng.next_u32())?
                                .parse::<IpAddr>()
                                .map_err(|e| {
                                    anyhow!("Failed to parse generated destination IP: {}", e)
                                })?
                                .into()
                        }
                    }
                });

                // Before the loop, let's retrieve the socket or whatever we need.
                let mut sock = {
                    match tech {
                        AfXdp(t) => {
                            t.sockets.get(i);
                        }
                    }
                };
            });

            if self.wait_for_finish {
                block_hdl.push(hdl);
            }
        }

        let logger = &ctx.logger;

        // Wait for threads to finish if needed.
        for hdl in block_hdl {
            hdl.join().map_err(|e| {
                logger
                    .blocking_read()
                    .log_msg(LogLevel::Error, &format!("Batch thread panicked: {:?}", e))
                    .ok();

                anyhow!("Batch thread panicked when joining: {:?}", e)
            })?;
        }

        Ok(())
    }
}
