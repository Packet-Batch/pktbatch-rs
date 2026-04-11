pub mod eth;
pub mod ip;

pub mod protocol;

pub mod payload;

use serde::{Deserialize, Serialize};

use crate::config::batch::{payload::PayloadOpts, protocol::ProtocolOpts};

#[derive(Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Batch {
    pub name: Option<String>,

    pub iface: Option<String>,

    pub wait_for_finish: bool,

    pub max_pkt: Option<u64>,
    pub max_byt: Option<u64>,

    pub duration: Option<u64>,
    pub send_interval: Option<u64>,

    pub thread_cnt: Option<u16>,

    pub protocol: ProtocolOpts,
    pub payload: PayloadOpts,
}
