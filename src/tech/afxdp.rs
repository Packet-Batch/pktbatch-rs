use crate::{
    config::tech::afxdp::TechAfXdp as TechAfXdpRaw,
    tech::{afxdp::opt::AfXdpOpts, base::TechBase},
};

pub mod opt;
pub mod socket;

#[derive(Clone)]
pub struct TechAfXdp {
    pub opts: AfXdpOpts,
}

impl TechBase {
    pub fn new_afxdp(opts: AfXdpOpts) -> Self {
        TechBase::AfXdp(TechAfXdp { opts })
    }

    pub fn is_afxdp(&self) -> bool {
        matches!(self, TechBase::AfXdp(_))
    }

    pub fn as_afxdp(&self) -> &TechAfXdp {
        let TechBase::AfXdp(afxdp) = self;

        afxdp
    }

    pub fn as_afxdp_mut(&mut self) -> &mut TechAfXdp {
        let TechBase::AfXdp(afxdp) = self;

        afxdp
    }
}

impl From<TechAfXdpRaw> for TechAfXdp {
    fn from(afxdp: TechAfXdpRaw) -> Self {
        Self {
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
