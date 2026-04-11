use serde::{Deserialize, Serialize};

#[derive(Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UdpOpts {
    pub src_port: Option<u16>,
    pub dst_port: Option<u16>,
}