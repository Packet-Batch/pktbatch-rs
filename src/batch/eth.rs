#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatchEth {
    src_mac: Option<[u8; 6]>,
    dst_mac: Option<[u8; 6]>,
}

