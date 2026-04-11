use crate::{batch::payload::Payload, batch::protocol::Protocol};

#[derive(Clone)]
pub struct BatchBase {
    pub id: u16,
    pub name: Option<String>,

    pub iface: String,

    pub wait_for_finish: bool,

    pub max_pkt: Option<u64>,
    pub max_byt: Option<u64>,

    pub duration: Option<u64>,
    pub send_interval: Option<u64>,
    
    pub thread_cnt: u16,

    pub protocol: Protocol,

    pub payload: Option<Payload>

}

pub type Batch = BatchBase;