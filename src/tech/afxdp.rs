use crate::tech::{afxdp::opt::AfXdpOpts, base::TechBase};

pub mod socket;
pub mod opt;

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