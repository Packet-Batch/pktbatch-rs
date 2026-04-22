use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use async_trait::async_trait;

use crate::{
    config::tech::Tech as TechCfg,
    context::Context,
    tech::{
        afxdp::{AfXdpDataInit, AfXdpDataThread, TechAfXdp, opt::AfXdpOpts},
        ext::TechExt,
    },
};

#[derive(Clone)]
pub enum TechBase {
    AfXdp(TechAfXdp),
}

pub type Tech = TechBase;

pub enum TechDataInit {
    AfXdp(AfXdpDataInit),
}

pub enum TechDataThread {
    AfXdp(AfXdpDataThread),
}

#[async_trait]
impl TechExt for TechBase {
    type Tech = TechBase;
    type Opts = ();

    type TechDataInit = TechDataInit;
    type TechDataThread = TechDataThread;

    fn new(_opts: Self::Opts) -> Self {
        unimplemented!("use From<TechCfg> instead")
    }

    fn get(&self) -> &Self::Tech {
        self
    }

    fn get_mut(&mut self) -> &mut Self::Tech {
        self
    }

    async fn init(
        &mut self,
        ctx: Context,
        iface_fb: Option<String>,
    ) -> Result<Option<Self::TechDataInit>> {
        match self {
            TechBase::AfXdp(t) => t
                .init(ctx, iface_fb)
                .await
                .map(|opt| opt.map(TechDataInit::AfXdp)),
        }
    }

    fn init_thread(
        &mut self,
        ctx: Context,
        thread_id: u16,
        iface_fb: Option<String>,
    ) -> Result<Option<Self::TechDataThread>> {
        match self {
            TechBase::AfXdp(t) => t
                .init_thread(ctx, thread_id, iface_fb)
                .map(|opt| opt.map(TechDataThread::AfXdp)),
        }
    }

    #[inline(always)]
    fn pkt_send(&mut self, pkt: &[u8], data_thread: Option<&mut Self::TechDataThread>) -> bool {
        match self {
            TechBase::AfXdp(t) => t.pkt_send(
                pkt,
                data_thread.map(|d| match d {
                    TechDataThread::AfXdp(d) => d,
                }),
            ),
        }
    }
}

impl From<TechCfg> for TechBase {
    fn from(tech: TechCfg) -> Self {
        match tech {
            TechCfg::AfXdp(opts) => TechBase::AfXdp(TechAfXdp {
                opts: AfXdpOpts {
                    queue_id: opts.queue_id,
                    need_wakeup: opts.need_wakeup,
                    shared_umem: opts.shared_umem,
                    batch_size: opts.batch_size,
                    zero_copy: opts.zero_copy,
                    sock_cnt: opts.sock_cnt,
                },
                sockets: Arc::new(HashMap::new()),
            }),
        }
    }
}
