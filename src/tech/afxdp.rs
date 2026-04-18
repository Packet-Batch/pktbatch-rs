pub mod opt;
pub mod socket;

use std::{
    collections::HashMap,
    ops::Deref,
    os::unix::thread,
    sync::{Arc, Mutex},
};

use anyhow::{Result, anyhow};
use async_trait::async_trait;

use crate::{
    config::tech::afxdp::TechAfXdpOpts as AfXdpOptsCfg,
    context::Context,
    tech::{
        afxdp::{
            opt::AfXdpOpts,
            socket::{XskTxConfig, XskTxSocket},
        },
        ext::TechExt,
    },
    util::{get_ifname_from_src_ip, sys::get_cpu_count},
};

#[derive(Clone, Default)]
pub struct TechAfXdp {
    pub opts: AfXdpOpts,
    pub sockets: Arc<HashMap<u16, Mutex<XskTxSocket>>>,
}

pub struct AfXdpData {
    pub socket: XskTxSocket,
}

#[async_trait]
impl TechExt for TechAfXdp {
    type Tech = TechAfXdp;
    type Opts = AfXdpOpts;
    type TechData = AfXdpData;

    fn new(opts: Self::Opts) -> Self {
        TechAfXdp {
            opts,
            sockets: Arc::new(HashMap::new()),
        }
    }

    fn get(&self) -> &Self::Tech {
        self
    }

    fn get_mut(&mut self) -> &mut Self::Tech {
        self
    }

    async fn init(&mut self, ctx: Context) -> anyhow::Result<()> {
        // We need to create sockets based off of thread count.
        let thread_cnt = if self.opts.thread_cnt > 0 {
            self.opts.thread_cnt
        } else {
            get_cpu_count() as u16
        };

        let cfg = &ctx.cfg;
        let logger = &ctx.logger;
        let batch = &ctx.batch;

        // Create tech from config.
        let tech: AfXdpOptsCfg = cfg
            .read()
            .await
            .tech
            .clone()
            .try_into()
            .map_err(|e| anyhow!("Failed to convert tech config for initialization: {}", e))?;

        // We need to retrieve the interface name.
        let if_name = if let Some(name) = tech.if_name {
            name
        } else {
            // We can use our util function to get the interface name from the source IP.
            let batch_guard = ctx.batch.read().await;

            let src_ip = batch_guard
                .batches
                .first()
                .and_then(|b| b.opt_ip.src.as_ref())
                .and_then(|src_vec| src_vec.first())
                .ok_or_else(|| anyhow!("No source IP found to derive interface name"))?;

            get_ifname_from_src_ip(src_ip)
                .ok_or_else(|| anyhow!("Could not find interface for IP {}", src_ip))?
        };

        // Create hash map for sockets.
        let mut sock_map = HashMap::new();

        for i in 0..thread_cnt {
            let queue_id = self.opts.queue_id.unwrap_or(i);
            let t_id = i + 1;

            // Create XSK socket.
            let xsk_cfg = XskTxConfig::from(self.opts.clone());

            xsk_cfg.if_name = if_name.clone();
            xsk_cfg.queue_id = queue_id;

            let socket = XskTxSocket::new(xsk_cfg)
                .map_err(|e| async {
                    // Log the error but continue trying to create sockets for other threads (don't want to fail the entire batch if one socket fails).
                    logger
                        .read()
                        .await
                        .log_msg(
                            LogLevel::Error,
                            &format!(
                                "Failed to create AF_XDP socket on queue ID {} (thread ID: {}): {}",
                                queue_id, t_id, e
                            ),
                        )
                        .ok();
                })
                .ok();

            if let Some(socket) = socket {
                sock_map.insert(i, Mutex::new(socket));
            }
        }

        self.sockets = Arc::new(sock_map);

        Ok(())
    }

    fn pkt_send(&mut self, ctx: Context, pkt: &[u8], data: Self::TechData) -> Result<()> {
        let mut sock = data.socket;

        sock.send_batch_single(pkt)
            .map_err(|e| anyhow!("failed to send packet: {}", e))?;

        Ok(())
    }
}

impl From<TechAfXdpRaw> for TechAfXdp {
    fn from(afxdp: TechAfXdpRaw) -> Self {
        Self {
            sockets: Arc::new(HashMap::new()),
            opts: AfXdpOpts::new(
                afxdp.queue_id,
                afxdp.need_wakeup,
                afxdp.shared_umem,
                afxdp.batch_size,
                afxdp.zero_copy,
            ),
        }
    }
}
