use crate::{config::tech::Tech as TechCfg, tech::afxdp::TechAfXdp};

#[derive(Clone)]
pub enum TechBase {
    AfXdp(TechAfXdp),
}

pub type Tech = TechBase;

impl From<TechCfg> for TechBase {
    fn from(tech: TechCfg) -> Self {
        if let Some(afxdp) = tech.afxdp {
            Self::AfXdp(afxdp.into())
        } else {
            unimplemented!()
        }
    }
}