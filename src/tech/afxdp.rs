pub mod opt;
pub mod socket;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use anyhow::{Result, anyhow};
use async_trait::async_trait;

use crate::{
    config::tech::{Tech as TechCfg, afxdp::TechAfXdpOpts as AfXdpOptsCfg},
    context::Context,
    tech::{
        afxdp::{
            opt::AfXdpOpts,
            socket::{XskTxConfig, XskTxSocket, XskUmem},
        },
        ext::TechExt,
    },
};

#[derive(Clone, Default)]
pub struct TechAfXdp {
    pub opts: AfXdpOpts,
    pub sockets: Arc<HashMap<u16, Mutex<XskTxSocket>>>,
}

pub struct AfXdpDataInit {
    pub none: bool,
}

pub struct AfXdpDataThread {
    pub socket: XskTxSocket,
}

#[async_trait]
impl TechExt for TechAfXdp {
    type Tech = TechAfXdp;
    type Opts = AfXdpOpts;

    type TechDataInit = AfXdpDataInit;
    type TechDataThread = AfXdpDataThread;

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

    async fn init(
        &mut self,
        _ctx: Context,
        _iface_fb: Option<String>,
    ) -> Result<Option<Self::TechDataInit>> {
        Ok(None)
    }

    fn init_thread(
        &mut self,
        ctx: Context,
        thread_id: u16,
        iface_fb: Option<String>,
    ) -> Result<Option<Self::TechDataThread>> {
        let cfg = &ctx.cfg;

        // Create tech from config.
        let tech = match cfg.blocking_read().tech.clone() {
            TechCfg::AfXdp(opts) => AfXdpOptsCfg::from(opts.clone()),
        };

        // We need to retrieve the interface name.
        let if_name = tech.if_name.clone().or(iface_fb).ok_or_else(|| {
            anyhow!("Failed to determine interface name for AF_XDP tech (missing in config and no fallback available)")
        })?;

        // Check if we need to create umem.
        let shared_umem = if self.opts.shared_umem {
            Some(
                XskUmem::new(&XskTxConfig::from(self.opts.clone())).map_err(|e| {
                    anyhow!("Failed to create shared UMEM for AF_XDP sockets: {}", e)
                })?,
            )
        } else {
            None
        };

        let queue_id = self.opts.queue_id.unwrap_or(thread_id);
        let t_id = thread_id + 1;

        // Create XSK socket.
        let mut xsk_cfg = XskTxConfig::from(self.opts.clone());

        xsk_cfg.if_name = if_name.clone();
        xsk_cfg.queue_id = queue_id;

        let sock = match XskTxSocket::new(xsk_cfg, shared_umem.as_ref()) {
            Ok(sock) => sock,
            Err(e) => {
                return Err(anyhow!(
                    "Failed to create AF_XDP socket for thread {}: {}",
                    t_id,
                    e
                ));
            }
        };

        Ok(Some(AfXdpDataThread { socket: sock }))
    }

    #[inline(always)]
    fn pkt_send(&mut self, pkt: &[u8], data_thread: Option<&mut Self::TechDataThread>) -> bool {
        let sock = match data_thread {
            Some(dt) => &mut dt.socket,
            None => return false,
        };

        match sock.send(pkt) {
            Ok(_) => true,
            Err(e) => {
                eprintln!("Failed to send packet on AF_XDP socket: {}", e);
                false
            }
        }
    }
}

impl From<AfXdpOptsCfg> for TechAfXdp {
    fn from(afxdp: AfXdpOptsCfg) -> Self {
        Self {
            sockets: Arc::new(HashMap::new()),
            opts: AfXdpOpts::new(
                afxdp.queue_id,
                afxdp.need_wakeup,
                afxdp.shared_umem,
                afxdp.batch_size,
                afxdp.zero_copy,
                afxdp.sock_cnt,
            ),
        }
    }
}
