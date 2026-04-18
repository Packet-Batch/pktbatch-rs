use anyhow::Result;
use async_trait::async_trait;

use crate::{
    config::tech::Tech as TechCfg,
    context::Context,
    tech::{
        afxdp::{AfXdpData, TechAfXdp, opt::AfXdpOpts},
        ext::TechExt,
    },
};

#[derive(Clone)]
pub enum TechBase {
    AfXdp(TechAfXdp),
}

pub type Tech = TechBase;

pub enum TechData {
    AfXdp(AfXdpData),
}

#[async_trait]
impl TechExt for TechBase {
    type Tech = TechBase;
    type Opts = ();
    type TechData = TechData;

    fn new(_opts: Self::Opts) -> Self {
        unimplemented!("use From<TechCfg> instead")
    }

    fn get(&self) -> &Self::Tech {
        self
    }

    fn get_mut(&mut self) -> &mut Self::Tech {
        self
    }

    async fn init(&mut self, ctx: Context) -> Result<()> {
        match self {
            TechBase::AfXdp(t) => t.init(ctx).await,
        }
    }

    fn pkt_send(&mut self, ctx: Context, pkt: &[u8], data: Self::TechData) -> Result<()> {
        match (self, data) {
            (TechBase::AfXdp(t), TechData::AfXdp(d)) => t.pkt_send(ctx, pkt, d),
            _ => anyhow::bail!("mismatched tech variant and data"),
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
                },
                sockets: Arc::new(HashMap::new()),
            }),
        }
    }
}
