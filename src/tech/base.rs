use crate::tech::afxdp::{TechAfXdp};

#[derive(Clone)]
pub enum TechBase {
    AfXdp(TechAfXdp),
}

pub type Tech = TechBase;

