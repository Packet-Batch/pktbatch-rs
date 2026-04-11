#[derive(Debug, Clone, PartialEq, Eq)]
enum TcpFlags {
    SYN = 1 << 0,
    ACK = 1 << 1,
    FIN = 1 << 2,
    RST = 1 << 3,
    PSH = 1 << 4,
    URG = 1 << 5,
    ECE = 1 << 6,
    CWR = 1 << 7,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolTcp {
    pub src_port: Option<u16>,
    pub dst_port: Option<u16>,

    pub flags: Option<u8>,
}