#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolIcmp {
    pub icmp_type: u8,
    pub icmp_code: u8,
}