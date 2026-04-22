use std::{
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use anyhow::{Result, anyhow};
use pnet::packet::{
    ethernet::MutableEthernetPacket, ipv4::MutableIpv4Packet, udp::MutableUdpPacket,
};

use std::sync::atomic::{AtomicBool, Ordering};

use crate::{
    batch::data::{
        BatchData,
        eth::ETH_HDR_LEN,
        ip::{FILL_FLAG_IP_DST, FILL_FLAG_IP_ID, FILL_FLAG_IP_SRC, FILL_FLAG_IP_TTL, IP_HDR_LEN},
        protocol::{FILL_FLAG_DST_PORT, FILL_FLAG_SRC_PORT, Protocol, ProtocolExt},
    },
    context::Context,
    logger::level::LogLevel,
    tech::{Tech, ext::TechExt},
    util::{get_cpu_count, get_cpu_rdtsc, get_ifname_from_src_ip},
};

const MAX_BUFFER_SZ: usize = 2048;

const OFF_START_IP_HDR: usize = ETH_HDR_LEN;
const OFF_START_PROTO_HDR: usize = ETH_HDR_LEN + IP_HDR_LEN;

struct RlData {
    pps: u64,
    bps: u64,
    next_update: Instant,
}

impl BatchData {
    pub async fn exec(
        &self,
        ctx: Context,
        id: u16,
        running: Arc<AtomicBool>,
        iface_fb: Option<String>,
    ) -> Result<()> {
        // Retrieve the number of threads we should create.
        let thread_cnt = if self.thread_cnt > 0 {
            self.thread_cnt
        } else {
            get_cpu_count() as u16
        };

        // Prepare block handles.
        let mut block_hdl = Vec::new();

        // Create rate limit context.
        // We need to do it outside of the threads for shared state.
        let rl_state = Arc::new(Mutex::new(RlData {
            pps: 0,
            bps: 0,
            next_update: Instant::now(),
        }));

        let core_ids = core_affinity::get_core_ids().unwrap();

        // Spawn threads.
        for i in 0..thread_cnt {
            let ctx = ctx.clone();
            let data = self.clone();
            let running = running.clone();
            let iface_fb = iface_fb.clone();

            let rl_state = rl_state.clone();

            let core_id = core_ids[i as usize % core_ids.len()];
            let hdl = thread::spawn(move || {
                core_affinity::set_for_current(core_id);

                // We'll want to clone immutable data here so that we aren't waiting for locks from shared threads (hurts performance).
                let mut tech = ctx.tech.blocking_read().clone();
                let logger = ctx.logger.blocking_read().clone();

                let batch = ctx.batch.blocking_read().clone();

                let data = data.clone();

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
                let if_name = match batch
                    .ovr_opts
                    .as_ref()
                    .and_then(|o| o.iface.clone())
                    .or_else(|| data.iface.clone().or_else(|| iface_fb.clone()))
                {
                    Some(if_name) => if_name,
                    None => {
                        logger
                            .log_msg(
                                LogLevel::Error,
                                &format!(
                                    "Failed to determine interface name for batch execution (batch_id={}, thread_id={})",
                                    id, i
                                ),
                            )
                            .ok();

                        return;
                    }
                };

                // Retrieve protocol from batch config.
                let proto: Protocol = Protocol::from(data.protocol.clone());

                let opt_ip = &data.opt_ip;

                // Retrieve a full list of source and destination IP addresses we'll be using.
                // We format these into the FullIpAddr structure.
                let src_ips = match data.opt_ip.get_src_ips(Some(&if_name)) {
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

                let dst_ips = match data.opt_ip.get_dst_ips() {
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

                // Generate seed using CPU timestamp counter for better randomness across threads.
                let mut seed = get_cpu_rdtsc() as u64;

                // Construct the packet buffer now.
                let mut buff: [u8; MAX_BUFFER_SZ] = [0; MAX_BUFFER_SZ];

                // Get protocol length.
                let proto_len = proto.get_hdr_len() as u16;

                let proto_hdr_end = OFF_START_PROTO_HDR + proto_len as usize;

                // Generate payload now so we know what the length is.
                let (pl_len, static_pl) = match data.payload {
                    Some(ref opt_pl) => match opt_pl.gen_payload(
                        &mut buff[proto_hdr_end..],
                        &mut seed,
                        proto_len as usize,
                    ) {
                        Ok(Some((len, is_static))) => (len, is_static),
                        Ok(None) => (0, true),
                        Err(e) => {
                            logger
                                .log_msg(
                                    LogLevel::Error,
                                    &format!("Failed to generate payload: {}", e),
                                )
                                .ok();

                            return;
                        }
                    },
                    None => (0, true),
                };

                // Determine full packet size now so we can use it as a boundry for filling header fields and such.
                let mut pkt_len =
                    ETH_HDR_LEN as u16 + IP_HDR_LEN as u16 + proto_len as u16 + pl_len;

                // Fill out ethernet header.
                // We use fill_init rom our eth options which is a helper func.
                match data
                    .opt_eth
                    .unwrap_or_default()
                    .fill_init(&mut buff[..ETH_HDR_LEN as usize], Some(if_name.clone()))
                {
                    Ok(_) => (),
                    Err(e) => {
                        logger
                            .log_msg(
                                LogLevel::Error,
                                &format!("Failed to fill Ethernet header: {}", e),
                            )
                            .ok();

                        return;
                    }
                }

                let (static_ip_src, static_ip_dst, static_ip_id, static_ip_ttl) = match data
                    .opt_ip
                    .fill_init(
                        &mut buff[OFF_START_IP_HDR..pkt_len as usize],
                        &mut seed,
                        &proto,
                        &src_ips,
                        &dst_ips,
                    ) {
                    Ok((src, dst, id, ttl)) => (src, dst, id, ttl),
                    Err(e) => {
                        logger
                            .log_msg(LogLevel::Error, &format!("Failed to fill IP header: {}", e))
                            .ok();

                        return;
                    }
                };

                // Now fill transport protocol header fields.
                let (static_proto_src, static_proto_dst) = match proto
                    .fill_init(&mut buff[OFF_START_PROTO_HDR..pkt_len as usize], &mut seed)
                {
                    Ok((src, dst)) => (src, dst),
                    Err(e) => {
                        logger
                            .log_msg(
                                LogLevel::Error,
                                &format!("Failed to fill protocol header: {}", e),
                            )
                            .ok();

                        return;
                    }
                };

                // Now determine flags for refills.
                let refill_ip_flags = {
                    let mut flags = 0;

                    if !static_ip_src {
                        flags |= FILL_FLAG_IP_SRC;
                    }

                    if !static_ip_dst {
                        flags |= FILL_FLAG_IP_DST;
                    }

                    if !static_ip_id {
                        flags |= FILL_FLAG_IP_ID;
                    }

                    if !static_ip_ttl {
                        flags |= FILL_FLAG_IP_TTL;
                    }

                    flags
                };

                let refill_proto_flags = {
                    let mut flags = 0;

                    if !static_proto_src {
                        flags |= FILL_FLAG_SRC_PORT;
                    }

                    if !static_proto_dst {
                        flags |= FILL_FLAG_DST_PORT;
                    }

                    flags
                };

                // Calculate checksums now.
                // We start with the transport layer.
                match proto.gen_checksum(&mut buff[ETH_HDR_LEN..pkt_len as usize]) {
                    Ok(_) => (),
                    Err(e) => {
                        logger
                            .log_msg(
                                LogLevel::Error,
                                &format!("Failed to generate protocol checksum: {}", e),
                            )
                            .ok();

                        return;
                    }
                }

                match opt_ip.gen_checksum(&mut buff[OFF_START_IP_HDR..]) {
                    Ok(_) => (),
                    Err(e) => {
                        logger
                            .log_msg(
                                LogLevel::Error,
                                &format!("Failed to generate IP checksum: {}", e),
                            )
                            .ok();

                        return;
                    }
                }

                // If we have a static payload + no refill flags, we don't need to recalculate checksums later on.
                let csum_not_needed = static_pl && refill_ip_flags == 0 && refill_proto_flags == 0;

                // Before the loop, setup tech specific thread data.
                let mut t_data = match tech.init_thread(ctx.clone(), i, iface_fb) {
                    Ok(opt) => opt,
                    Err(e) => {
                        logger
                            .log_msg(
                                LogLevel::Error,
                                &format!("Failed to initialize tech thread data: {}", e),
                            )
                            .ok();

                        return;
                    }
                };

                let start_time = Instant::now();

                let to_end_time = {
                    if let Some(dur) = data.duration {
                        Some(Duration::from_secs(dur))
                    } else {
                        None
                    }
                };

                // Counters for total packets and bytes sent by this thread.
                let mut cur_pkts = 0;
                let mut cur_byts = 0;

                // Determine limits.
                // Determine the max packet and bytes for this thread if applicable.
                let max_pkt_cnt = {
                    if let Some(max_pkt) = data.max_pkt {
                        Some((max_pkt / thread_cnt as u64).max(1))
                    } else {
                        None
                    }
                };

                let max_byt_cnt = {
                    if let Some(max_byt) = data.max_byt {
                        Some((max_byt / thread_cnt as u64).max(1))
                    } else {
                        None
                    }
                };

                let pps = if let Some(pps) = data.pps {
                    Some(pps)
                } else {
                    None
                };

                let bps = if let Some(bps) = data.bps {
                    Some(bps)
                } else {
                    None
                };

                loop {
                    // Check if we need to stop execution (from main thread signal).
                    if !running.load(Ordering::Relaxed) {
                        break;
                    }

                    // Check if we've reached the configured duration.
                    if let Some(max_dur) = to_end_time {
                        if Instant::now().duration_since(start_time) >= max_dur {
                            logger
                            .log_msg(
                                LogLevel::Info,
                                &format!(
                                    "[B{}][T{}] Finished execution after reaching max duration of {:?}",
                                    id, i, max_dur
                                ),
                            ).ok();

                            break;
                        }
                    }

                    // Check for max packets.
                    if let Some(max_pk) = max_pkt_cnt {
                        cur_pkts += 1;

                        if cur_pkts >= max_pk {
                            logger
                                .log_msg(
                                    LogLevel::Info,
                                    &format!("[B{}][T{}] Finished execution after sending max packets of {}", i, id, max_pk),
                                )
                                .ok();

                            break;
                        }
                    }

                    // Check for max bytes.
                    if let Some(max_by) = max_byt_cnt {
                        cur_byts += pkt_len as u64;

                        if cur_byts >= max_by {
                            logger
                                .log_msg(
                                    LogLevel::Info,
                                    &format!("[B{}][T{}] Finished execution after sending max bytes of {}", i, id, max_by),
                                )
                                .ok();

                            break;
                        }
                    }

                    // Check for per-second limits.
                    if pps.is_some() || bps.is_some() {
                        let mut rl = rl_state.lock().unwrap();

                        let now = Instant::now();

                        if now >= rl.next_update {
                            // Reset counters and determine next update time.
                            rl.pps = 0;
                            rl.bps = 0;
                            rl.next_update = now + Duration::from_secs(1);
                        } else {
                            // Check if sending the packet would exceed the limits.
                            if let Some(pps_limit) = pps {
                                if rl.pps >= pps_limit {
                                    let sleep_dur = rl.next_update.duration_since(now);

                                    thread::sleep(sleep_dur);

                                    continue;
                                }
                            }

                            if let Some(bps_limit) = bps {
                                if rl.bps + pkt_len as u64 > bps_limit {
                                    let sleep_dur = rl.next_update.duration_since(now);

                                    thread::sleep(sleep_dur);

                                    continue;
                                }
                            }
                        }

                        // If we reach here, it means we can send the packet without exceeding limits. Update counters accordingly.
                        rl.pps += 1;
                        rl.bps += pkt_len as u64;
                    }

                    // Check if we need to regenerate the payload.
                    if !static_pl {
                        match data.payload {
                            Some(ref opt_pl) => {
                                let old_len = pkt_len;

                                match opt_pl.gen_payload(
                                    &mut buff[OFF_START_PROTO_HDR + proto_len as usize..],
                                    &mut seed,
                                    proto_len as usize,
                                ) {
                                    Ok(Some((len, _))) => {
                                        // Update packet length accordingly.
                                        pkt_len = ETH_HDR_LEN as u16
                                            + IP_HDR_LEN as u16
                                            + proto_len as u16
                                            + len;
                                    }
                                    Ok(None) => {
                                        pkt_len = ETH_HDR_LEN as u16
                                            + IP_HDR_LEN as u16
                                            + proto_len as u16;
                                    }
                                    Err(e) => {
                                        logger
                                            .log_msg(
                                                LogLevel::Error,
                                                &format!("Failed to regenerate payload: {}", e),
                                            )
                                            .ok();

                                        continue;
                                    }
                                }

                                if pkt_len != old_len {
                                    logger
                                        .log_msg(
                                            LogLevel::Debug,
                                            &format!(
                                                "Regenerated payload with new length {} bytes (old length was {} bytes)",
                                                pkt_len - ETH_HDR_LEN as u16 - IP_HDR_LEN as u16 - proto_len as u16,
                                                old_len - ETH_HDR_LEN as u16 - IP_HDR_LEN as u16 - proto_len as u16
                                            ),
                                        )
                                        .ok();

                                    let mut iph = match MutableIpv4Packet::new(
                                        &mut buff[OFF_START_IP_HDR..pkt_len as usize],
                                    ) {
                                        Some(p) => p,
                                        None => {
                                            logger
                                                        .log_msg(
                                                            LogLevel::Error,
                                                            &format!(
                                                                "Failed to create mutable IPv4 packet for payload regeneration"
                                                            ),
                                                        )
                                                        .ok();

                                            continue;
                                        }
                                    };

                                    iph.set_total_length(pkt_len - ETH_HDR_LEN as u16);

                                    // Now set protocol length.
                                    match proto.set_total_len(
                                        &mut buff[OFF_START_PROTO_HDR..pkt_len as usize],
                                        pkt_len - ETH_HDR_LEN as u16 - IP_HDR_LEN as u16,
                                    ) {
                                        Ok(_) => (),
                                        Err(e) => {
                                            logger
                                                .log_msg(
                                                    LogLevel::Error,
                                                    &format!(
                                                        "Failed to set protocol total length: {}",
                                                        e
                                                    ),
                                                )
                                                .ok();

                                            continue;
                                        }
                                    }
                                }
                            }
                            None => (),
                        }
                    }

                    // Check if we need to refill the IP header at all.
                    if refill_ip_flags != 0 {
                        if let Err(e) = opt_ip.fill(
                            &mut buff[ETH_HDR_LEN..pkt_len as usize],
                            refill_ip_flags,
                            &mut seed,
                            &src_ips,
                            &dst_ips,
                        ) {
                            logger
                                .log_msg(
                                    LogLevel::Error,
                                    &format!("Failed to refill IP header: {}", e),
                                )
                                .ok();

                            continue;
                        }
                    }

                    // Check if we need to refill the protocol header at all.
                    if refill_proto_flags != 0 {
                        if let Err(e) = proto.fill(
                            &mut buff[(ETH_HDR_LEN + IP_HDR_LEN)..],
                            refill_proto_flags,
                            &mut seed,
                        ) {
                            logger
                                .log_msg(
                                    LogLevel::Error,
                                    &format!("Failed to refill protocol header: {}", e),
                                )
                                .ok();

                            continue;
                        }
                    }

                    // Recalculate checksums if needed.
                    if !csum_not_needed {
                        // We start with the transport layer.
                        if let Err(e) = proto.gen_checksum(&mut buff[ETH_HDR_LEN..pkt_len as usize])
                        {
                            logger
                                .log_msg(
                                    LogLevel::Error,
                                    &format!("Failed to regenerate protocol checksum: {}", e),
                                )
                                .ok();

                            continue;
                        }

                        if let Err(e) = opt_ip.gen_checksum(&mut buff[OFF_START_IP_HDR..]) {
                            logger
                                .log_msg(
                                    LogLevel::Error,
                                    &format!("Failed to regenerate IP checksum: {}", e),
                                )
                                .ok();

                            continue;
                        }
                    }

                    // Attempt to send packet immediately.
                    // First run should have all fields set regardless.
                    match tech.pkt_send(&buff[..pkt_len as usize], t_data.as_mut()) {
                        true => (),
                        false => {
                            logger
                                .log_msg(LogLevel::Error, &format!("Failed to send packet"))
                                .ok();

                            continue;
                        }
                    }

                    // Check if we need to sleep between sends based on batch config.
                    if let Some(interval) = data.send_interval {
                        thread::sleep(Duration::from_micros(interval));
                    }
                }
            });

            if self.wait_for_finish || i == 0 {
                block_hdl.push(hdl);
            }
        }

        let logger = &ctx.logger;

        // Wait for threads to finish if needed.
        for hdl in block_hdl {
            match hdl.join() {
                Ok(_) => (),
                Err(e) => {
                    logger
                        .read()
                        .await
                        .log_msg(LogLevel::Error, &format!("Batch thread panicked: {:?}", e))
                        .ok();

                    return Err(anyhow!("Batch thread panicked when joining: {:?}", e));
                }
            }
        }

        Ok(())
    }
}
