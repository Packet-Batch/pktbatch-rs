#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IpSourceRange {
    pub net_ip: [u8; 4],
    pub net_mask: [u8; 4],
}