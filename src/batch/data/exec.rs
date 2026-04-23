pub mod data;

use std::{
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use anyhow::{Result, anyhow};

use std::sync::atomic::{AtomicBool, Ordering};

use crate::{
    batch::data::{
        BatchData,
        eth::ETH_HDR_LEN,
        exec::data::{
            CsumCalcFail, GenPlFail, LimitFail, MAX_BUFFER_SZ, OFF_START_IP_HDR,
            OFF_START_PROTO_HDR, PktInspectFail, TechExecData,
        },
        ip::{FILL_FLAG_IP_DST, FILL_FLAG_IP_ID, FILL_FLAG_IP_SRC, FILL_FLAG_IP_TTL, IP_HDR_LEN},
        protocol::{FILL_FLAG_DST_PORT, FILL_FLAG_SRC_PORT, Protocol, ProtocolExt},
    },
    context::Context,
    logger::level::LogLevel,
    tech::ext::TechExt,
    util::{get_cpu_count, get_cpu_rdtsc},
};

pub struct RlData {
    pps: u64,
    bps: u64,
    next_update: Instant,
}

impl BatchData {
    pub fn exec_prepare(&self, ctx: Context, iface_fb: Option<String>) -> Result<TechExecData> {
        let data = self.clone();
        let iface_fb = iface_fb.clone();

        let batch = &ctx.batch;

        // We need to retrieve the interface name.
        let iface = match batch
            .blocking_read()
            .ovr_opts
            .as_ref()
            .and_then(|o| o.iface.clone())
            .or_else(|| data.iface.clone().or_else(|| iface_fb.clone()))
        {
            Some(if_name) => if_name,
            None => {
                return Err(anyhow!("Failed to determine interface name for batch"));
            }
        };

        // Retrieve protocol from batch config.
        let protocol = Protocol::from(data.protocol.clone());

        let opt_ip = &data.opt_ip;

        // Retrieve a full list of source and destination IP addresses we'll be using.
        // We format these into the FullIpAddr structure.
        let src_ips = match data.opt_ip.get_src_ips(Some(&iface)) {
            Ok(ips) => ips,
            Err(e) => return Err(anyhow!("Failed to retrieve source IP addresses: {}", e)),
        };

        let dst_ips = match data.opt_ip.get_dst_ips() {
            Ok(ips) => ips,
            Err(e) => {
                return Err(anyhow!(
                    "Failed to retrieve destination IP addresses: {}",
                    e
                ));
            }
        };

        // Generate seed using CPU timestamp counter for better randomness across threads.
        let mut seed = get_cpu_rdtsc() as u64;

        // Construct the packet buffer now.
        let mut buff: [u8; MAX_BUFFER_SZ] = [0; MAX_BUFFER_SZ];

        // Get protocol length.
        let proto_len = protocol.get_hdr_len() as u16;

        let proto_hdr_end = OFF_START_PROTO_HDR + proto_len as usize;

        // Generate payload now so we know what the length is.
        let (pl_len, static_pl) = match data.payload {
            Some(ref opt_pl) => {
                match opt_pl.gen_payload(&mut buff[proto_hdr_end..], &mut seed, proto_len as usize)
                {
                    Ok(Some((len, is_static))) => (len, is_static),
                    Ok(None) => (0, true),
                    Err(e) => return Err(anyhow!("Failed to generate payload: {}", e)),
                }
            }
            None => (0, true),
        };

        // Determine full packet size now so we can use it as a boundry for filling header fields and such.
        let pkt_len = ETH_HDR_LEN as u16 + IP_HDR_LEN as u16 + proto_len as u16 + pl_len;

        // Fill out ethernet header.
        // We use fill_init rom our eth options which is a helper func.
        match data
            .opt_eth
            .unwrap_or_default()
            .fill_init(&mut buff[..ETH_HDR_LEN as usize], Some(iface.clone()))
        {
            Ok(_) => (),
            Err(e) => return Err(anyhow!("Failed to fill Ethernet header: {}", e)),
        }

        let (static_ip_src, static_ip_dst, static_ip_id, static_ip_ttl) =
            match data.opt_ip.fill_init(
                &mut buff[OFF_START_IP_HDR..pkt_len as usize],
                &mut seed,
                &protocol,
                &src_ips,
                &dst_ips,
            ) {
                Ok((src, dst, id, ttl)) => (src, dst, id, ttl),
                Err(e) => return Err(anyhow!("Failed to fill IP header: {}", e)),
            };

        // Now fill transport protocol header fields.
        let (static_proto_src, static_proto_dst) =
            match protocol.fill_init(&mut buff[OFF_START_PROTO_HDR..pkt_len as usize], &mut seed) {
                Ok((src, dst)) => (src, dst),
                Err(e) => return Err(anyhow!("Failed to fill protocol header: {}", e)),
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
        match protocol.gen_checksum(&mut buff[ETH_HDR_LEN..pkt_len as usize]) {
            Ok(_) => (),
            Err(e) => return Err(anyhow!("Failed to generate protocol checksum: {}", e)),
        }

        match opt_ip.gen_checksum(&mut buff[OFF_START_IP_HDR..]) {
            Ok(_) => (),
            Err(e) => return Err(anyhow!("Failed to generate IP checksum: {}", e)),
        }

        // If we have a static payload + no refill flags, we don't need to recalculate checksums later on.
        let csum_not_needed = static_pl && refill_ip_flags == 0 && refill_proto_flags == 0;

        Ok(TechExecData {
            iface,
            src_ips,
            dst_ips,
            protocol,
            proto_len,
            proto_hdr_end,
            seed,
            pkt_len,
            buff,
            refill_ip_flags,
            refill_proto_flags,
            csum_not_needed,
            pl_len,
            static_pl,
            max_pkt_cnt: data.max_pkt,
            max_byt_cnt: data.max_byt,
            pps: data.pps,
            bps: data.bps,
            start_time: Instant::now(),
            to_end_time: None,
            cur_pkts: 0,
            cur_byts: 0,
        })
    }

    pub async fn exec(
        &self,
        ctx: Context,
        id: u16,
        running: Arc<AtomicBool>,
        iface_fb: Option<String>,
    ) -> Result<()> {
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

                let data = data.clone();

                // Retrieve execution data and also initialize everything for local thread data.
                let mut e_data = match data.exec_prepare(ctx.clone(), iface_fb.clone()) {
                    Ok(data) => data,
                    Err(e) => {
                        logger
                            .log_msg(
                                LogLevel::Error,
                                &format!("Failed to prepare execution data: {} (batch_id={}, thread_id={})", e, id, i),
                            )
                            .ok();

                        return;
                    }
                };

                logger
                    .log_msg(
                        LogLevel::Info,
                        &format!(
                            "Starting batch execution (batch_id={}, thread_id={})",
                            id, i
                        ),
                    )
                    .ok();

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

                loop {
                    // Check if we need to stop execution (from main thread signal).
                    if !running.load(Ordering::Relaxed) {
                        break;
                    }

                    // Check for limits.
                    match e_data.check_limits(rl_state.clone()) {
                        LimitFail::None => (),
                        fail => {
                            logger
                                .log_msg(
                                    LogLevel::Info,
                                    &format!("Stopping batch execution: {}", fail),
                                )
                                .ok();

                            break;
                        }
                    }

                    // Check if we need to regenerate the payload.
                    match e_data.check_and_gen_pl(ctx.clone(), &data) {
                        GenPlFail::None => (),
                        fail => {
                            logger
                                .log_msg(
                                    LogLevel::Error,
                                    &format!("Failed to generate payload for packet: {}", fail),
                                )
                                .ok();

                            continue;
                        }
                    }

                    // Check for packet inspection changes.
                    match e_data.check_packet_inspection(&data) {
                        PktInspectFail::None => (),
                        fail => {
                            logger
                                .log_msg(
                                    LogLevel::Error,
                                    &format!("Failed during packet inspection check: {}", fail),
                                )
                                .ok();

                            continue;
                        }
                    }

                    // Recalculate checksums if needed.
                    match e_data.calc_checksum() {
                        CsumCalcFail::None => (),
                        fail => {
                            logger
                                .log_msg(
                                    LogLevel::Error,
                                    &format!("Failed to calculate checksum for packet: {}", fail),
                                )
                                .ok();

                            continue;
                        }
                    }

                    // Attempt to send packet immediately.
                    // First run should have all fields set regardless.
                    match tech.pkt_send(&e_data.buff[..e_data.pkt_len as usize], t_data.as_mut()) {
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
