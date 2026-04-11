pub mod afxdp;

use serde::{Deserialize, Serialize};

use crate::{config::tech::afxdp::TechAfXdp};

#[derive(Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tech {
    pub afxdp: Option<TechAfXdp>,
}