use std::{
    fmt,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use pnet::packet::ipv4::MutableIpv4Packet;

use crate::{
    batch::data::{
        BatchData,
        eth::ETH_HDR_LEN,
        exec::RlData,
        ip::{FullIpAddr, IP_HDR_LEN},
        protocol::Protocol,
    },
    context::Context,
    logger::level::LogLevel,
};

use crate::batch::data::protocol::ProtocolExt;

pub const MAX_BUFFER_SZ: usize = 2048;

pub const OFF_START_IP_HDR: usize = ETH_HDR_LEN;
pub const OFF_START_PROTO_HDR: usize = ETH_HDR_LEN + IP_HDR_LEN;

#[derive(Clone, Debug)]
pub struct TechExecData {
    pub iface: String,

    pub src_ips: Vec<FullIpAddr>,
    pub dst_ips: Vec<FullIpAddr>,

    pub protocol: Protocol,
    pub proto_len: u16,
    pub proto_hdr_end: usize,

    pub seed: u64,

    pub pkt_len: u16,
    pub buff: [u8; MAX_BUFFER_SZ],

    pub refill_ip_flags: u32,
    pub refill_proto_flags: u32,
    pub csum_not_needed: bool,

    pub pl_len: u16,
    pub static_pl: bool,

    pub max_pkt_cnt: Option<u64>,
    pub max_byt_cnt: Option<u64>,
    pub pps: Option<u64>,
    pub bps: Option<u64>,

    pub start_time: Instant,
    pub to_end_time: Option<Duration>,

    pub cur_pkts: u64,
    pub cur_byts: u64,
}

pub enum LimitFail {
    PktCnt,
    BytCnt,
    Time,
    Pps,
    Bps,
    None,
}

impl fmt::Display for LimitFail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LimitFail::PktCnt => write!(f, "packet count limit reached"),

            LimitFail::BytCnt => write!(f, "byte count limit reached"),

            LimitFail::Pps => write!(f, "packets per second limit reached"),
            LimitFail::Bps => write!(f, "bits per second limit reached"),

            LimitFail::Time => write!(f, "time limit reached"),

            LimitFail::None => write!(f, "no limits reached"),
        }
    }
}

pub enum GenPlFail {
    Generate,
    Iph,
    Proto,
    None,
}

impl fmt::Display for GenPlFail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GenPlFail::Generate => write!(f, "failed to generate static payload"),

            GenPlFail::Iph => write!(
                f,
                "static payload generation failed due to IP header configuration"
            ),

            GenPlFail::Proto => write!(
                f,
                "static payload generation failed due to protocol header configuration"
            ),

            GenPlFail::None => write!(f, "no static payload generation failure"),
        }
    }
}

#[derive(Eq, PartialEq)]
pub enum PktInspectFail {
    Ip,
    Protocol,
    None,
}

impl fmt::Display for PktInspectFail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PktInspectFail::Ip => {
                write!(f, "packet inspection failed during IP header filling")
            }

            PktInspectFail::Protocol => {
                write!(f, "packet inspection failed during protocol header filling")
            }
            PktInspectFail::None => write!(f, "no packet inspection failure"),
        }
    }
}

#[derive(Eq, PartialEq)]
pub enum CsumCalcFail {
    Ip,
    Protocol,
    None,
}

impl fmt::Display for CsumCalcFail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CsumCalcFail::Ip => write!(f, "checksum calculation failed for IP header"),

            CsumCalcFail::Protocol => write!(f, "checksum calculation failed for protocol header"),

            CsumCalcFail::None => write!(f, "no checksum calculation failure"),
        }
    }
}

impl TechExecData {
    /// Checks for all limits. This includes max packet count, max byte count, time limit, and per-second limits (PPS and BPS).
    ///
    /// # Arguments
    /// * `rl_state` - The current rate limit state, which contains the current PPS and BPS counters and the next update time for per-second limits.
    ///
    /// # Returns
    /// A `LimitFail` enum indicating which limit was exceeded, or `LimitFail::None` if no limits were exceeded.
    #[inline(always)]
    pub fn check_limits(&self, rl_state: Arc<Mutex<RlData>>) -> LimitFail {
        // Check max packet count limit.
        if !self.limit_max_pkt_cnt() {
            return LimitFail::PktCnt;
        }

        // Check max byte count limit.
        if !self.limit_max_byt_cnt() {
            return LimitFail::BytCnt;
        }

        // Check for time limit exceedance.
        if !self.limit_time() {
            return LimitFail::Time;
        }

        // Finally, check for per-second limits (PPS and BPS).
        // We need to pass rl_state in this case.
        self.limit_per_sec(rl_state)
    }

    /// Checks if the max packet count limit has been exceeded.
    #[inline(always)]
    fn limit_max_pkt_cnt(&self) -> bool {
        if let Some(max_pkt_cnt) = self.max_pkt_cnt {
            if self.cur_pkts >= max_pkt_cnt {
                return false;
            }
        }

        true
    }

    /// Checks if the max byte count limit has been exceeded.
    #[inline(always)]
    fn limit_max_byt_cnt(&self) -> bool {
        if let Some(max_byt_cnt) = self.max_byt_cnt {
            if self.cur_byts >= max_byt_cnt {
                return false;
            }
        }

        true
    }

    /// Checks if the time limit has been exceeded.
    #[inline(always)]
    fn limit_time(&self) -> bool {
        if let Some(max_dur) = self.to_end_time {
            if Instant::now().duration_since(self.start_time) >= max_dur {
                return false;
            }
        }

        true
    }

    /// Checks if the per-second limits (PPS and BPS) have been exceeded. This function also updates the PPS and BPS counters in the `RlData` state.
    ///
    /// # Arguments
    /// * `rl_state` - The current rate limit state, which contains the current PPS and BPS counters and the next update time for per-second limits.
    ///
    /// # Returns
    /// A `LimitFail` enum indicating which per-second limit was exceeded, or `LimitFail::None` if no per-second limits were exceeded.
    #[inline(always)]
    fn limit_per_sec(&self, rl_state: Arc<Mutex<RlData>>) -> LimitFail {
        if self.pps.is_some() || self.bps.is_some() {
            let mut rl = rl_state.lock().unwrap();

            let now = Instant::now();

            if now >= rl.next_update {
                // Reset counters and determine next update time.
                rl.pps = 0;
                rl.bps = 0;
                rl.next_update = now + Duration::from_secs(1);
            } else {
                // Check if sending the packet would exceed the limits.
                if let Some(pps_limit) = self.pps {
                    if rl.pps >= pps_limit {
                        return LimitFail::Pps;
                    }
                }

                if let Some(bps_limit) = self.bps {
                    if rl.bps + self.pkt_len as u64 > bps_limit {
                        return LimitFail::Bps;
                    }
                }
            }

            // If we reach here, it means we can send the packet without exceeding limits. Update counters accordingly.
            rl.pps += 1;
            rl.bps += self.pkt_len as u64;
        }

        LimitFail::None
    }

    /// Checks and regenerates payload if payload isn't static.
    ///
    /// # Arguments
    /// * `ctx` - The application context, which may contain shared data and resources (e.g., logger).
    /// * `data` - The batch data containing the payload configuration and generation logic.
    ///
    /// # Returns
    /// A `GenPlFail` enum indicating if there was a failure during payload generation, or `GenPlFail::None` if there was no failure.
    #[inline(always)]
    pub fn check_and_gen_pl(&mut self, ctx: Context, data: &BatchData) -> GenPlFail {
        // Check if we need to regenerate the payload.
        if !self.static_pl {
            match data.payload {
                Some(ref opt_pl) => {
                    let old_len = self.pkt_len;

                    match opt_pl.gen_payload(
                        &mut self.buff[OFF_START_PROTO_HDR + self.proto_len as usize..],
                        &mut self.seed,
                        self.proto_len as usize,
                    ) {
                        Ok(Some((len, _))) => {
                            // Update packet length accordingly.
                            self.pkt_len = ETH_HDR_LEN as u16
                                + IP_HDR_LEN as u16
                                + self.proto_len as u16
                                + len;
                        }
                        Ok(None) => {
                            self.pkt_len =
                                ETH_HDR_LEN as u16 + IP_HDR_LEN as u16 + self.proto_len as u16;
                        }
                        Err(_) => return GenPlFail::Generate,
                    }

                    if self.pkt_len != old_len {
                        ctx.logger.blocking_read()
                                        .log_msg(
                                            LogLevel::Debug,
                                            &format!(
                                                "Regenerated payload with new length {} bytes (old length was {} bytes)",
                                                self.pkt_len - ETH_HDR_LEN as u16 - IP_HDR_LEN as u16 - self.proto_len as u16,
                                                old_len - ETH_HDR_LEN as u16 - IP_HDR_LEN as u16 - self.proto_len as u16
                                            ),
                                        )
                                        .ok();

                        let mut iph = match MutableIpv4Packet::new(
                            &mut self.buff[OFF_START_IP_HDR..self.pkt_len as usize],
                        ) {
                            Some(p) => p,
                            None => return GenPlFail::Iph,
                        };

                        iph.set_total_length(self.pkt_len - ETH_HDR_LEN as u16);

                        // Now set protocol length.
                        match self.protocol.set_total_len(
                            &mut self.buff[OFF_START_PROTO_HDR..self.pkt_len as usize],
                            self.pkt_len - ETH_HDR_LEN as u16 - IP_HDR_LEN as u16,
                        ) {
                            Ok(_) => (),
                            Err(_) => return GenPlFail::Proto,
                        }
                    }
                }
                None => (),
            }
        }

        GenPlFail::None
    }

    /// Checks and sets in-depth packet fields. This includes filling in IP header fields and protocol header fields based on the configuration and generation logic in `BatchData`.
    ///
    /// # Arguments
    /// * `data` - The batch data containing the configuration and generation logic for IP and protocol headers.
    ///
    /// # Returns
    /// A `PktInspectFail` enum indicating if there was a failure during packet inspection and filling, or `PktInspectFail::None` if there was no failure.
    #[inline(always)]
    pub fn check_packet_inspection(&mut self, data: &BatchData) -> PktInspectFail {
        // First check IP header filling if needed.
        let ip_inspect_result = self.pkt_inspection_ip(data);

        if ip_inspect_result != PktInspectFail::None {
            return ip_inspect_result;
        }

        // Next, check protocol header filling if needed.
        let proto_inspect_result = self.pkt_inspection_protocol();

        if proto_inspect_result != PktInspectFail::None {
            return proto_inspect_result;
        }

        PktInspectFail::None
    }

    /// Checks and calculates checksums for IP and protocol headers if needed.
    ///
    /// # Arguments
    /// * `data` - The batch data containing the configuration for checksum calculation.
    ///
    /// # Returns
    /// A `CsumCalcFail` enum indicating if there was a failure during checksum calculation, or `CsumCalcFail::None` if there was no failure.
    #[inline(always)]
    fn pkt_inspection_ip(&mut self, data: &BatchData) -> PktInspectFail {
        if self.refill_ip_flags != 0 {
            if let Err(_) = data.opt_ip.fill(
                &mut self.buff[ETH_HDR_LEN..self.pkt_len as usize],
                self.refill_ip_flags,
                &mut self.seed,
                &self.src_ips,
                &self.dst_ips,
            ) {
                return PktInspectFail::Ip;
            }
        }

        PktInspectFail::None
    }

    /// Checks and fills protocol header fields if needed.
    ///
    /// # Returns
    /// A `PktInspectFail` enum indicating if there was a failure during protocol header filling, or `PktInspectFail::None` if there was no failure.
    #[inline(always)]
    fn pkt_inspection_protocol(&mut self) -> PktInspectFail {
        if self.refill_proto_flags != 0 {
            if let Err(_) = self.protocol.fill(
                &mut self.buff[OFF_START_PROTO_HDR..self.pkt_len as usize],
                self.refill_proto_flags,
                &mut self.seed,
            ) {
                return PktInspectFail::Protocol;
            }
        }

        PktInspectFail::None
    }

    /// Checks and calculates checksums for IP and protocol headers if needed.
    ///
    /// # Returns
    /// A `CsumCalcFail` enum indicating if there was a failure during checksum calculation, or `CsumCalcFail::None` if there was no failure.
    #[inline(always)]
    pub fn calc_checksum(&mut self) -> CsumCalcFail {
        if self.csum_not_needed {
            return CsumCalcFail::None;
        }

        // First, calculate protocol header checksum if needed.
        let proto_csum_result = self.calc_checksum_protocol();

        if proto_csum_result != CsumCalcFail::None {
            return proto_csum_result;
        }

        // Next, calculate IP header checksum if needed.
        let ip_csum_result = self.calc_checksum_ip();

        if ip_csum_result != CsumCalcFail::None {
            return ip_csum_result;
        }

        CsumCalcFail::None
    }

    /// Checks and calculates checksum for protocol header if needed.
    ///
    /// # Returns
    /// A `CsumCalcFail` enum indicating if there was a failure during protocol header checksum calculation, or `CsumCalcFail::None` if there was no failure.
    #[inline(always)]
    fn calc_checksum_protocol(&mut self) -> CsumCalcFail {
        if let Err(_) = self
            .protocol
            .gen_checksum(&mut self.buff[ETH_HDR_LEN..self.pkt_len as usize])
        {
            return CsumCalcFail::Protocol;
        }

        CsumCalcFail::None
    }

    /// Checks and calculates checksum for IP header if needed.
    ///
    /// # Returns
    /// A `CsumCalcFail` enum indicating if there was a failure during IP header checksum calculation, or `CsumCalcFail::None` if there was no failure.
    #[inline(always)]
    fn calc_checksum_ip(&mut self) -> CsumCalcFail {
        if let Err(_) = self
            .protocol
            .gen_checksum(&mut self.buff[OFF_START_IP_HDR..self.pkt_len as usize])
        {
            return CsumCalcFail::Ip;
        }

        CsumCalcFail::None
    }
}
