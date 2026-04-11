use anyhow::Result;

use crate::{
    batch::{eth::EthOpts, ip::IpOpts, payload::PayloadOpts, protocol::ProtocolOpts},
    config::batch::Batch as BatchCfg,
    util::sys::get_cpu_count,
};

#[derive(Clone, Default)]
pub struct BatchBase {
    pub id: u16,
    pub name: Option<String>,

    pub iface: Option<String>,

    pub wait_for_finish: bool,

    pub max_pkt: Option<u64>,
    pub max_byt: Option<u64>,

    pub duration: Option<u64>,
    pub send_interval: Option<u64>,

    pub thread_cnt: u16,

    pub protocol: ProtocolOpts,

    pub opts_eth: Option<EthOpts>,
    pub opts_ip: IpOpts,
    pub protocol_opts: Option<ProtocolOpts>,

    pub payload: Option<PayloadOpts>,
}

pub type Batch = BatchBase;

impl BatchBase {
    pub fn new(
        id: u16,
        name: Option<String>,
        iface: Option<String>,
        wait_for_finish: bool,
        max_pkt: Option<u64>,
        max_byt: Option<u64>,
        duration: Option<u64>,
        send_interval: Option<u64>,
        thread_cnt: u16,
        protocol: ProtocolOpts,
        payload: Option<PayloadOpts>,
    ) -> Self {
        Self {
            id,
            name,
            iface,
            wait_for_finish,
            max_pkt,
            max_byt,
            duration,
            send_interval,
            thread_cnt,
            protocol,
            payload,
            opts_eth: None,
            opts_ip: IpOpts::default(),
            protocol_opts: None,
        }
    }

    pub fn start(&mut self) -> Result<()> {
        Ok(())
    }

    pub fn stop(&mut self) -> Result<()> {
        Ok(())
    }
}

impl From<BatchCfg> for BatchBase {
    fn from(cfg: BatchCfg) -> Self {
        // Retrieve thread count.
        // We use core count if none is specified.
        let thread_cnt = cfg.thread_cnt.unwrap_or(get_cpu_count() as u16).max(1);

        Self::new(
            0,
            cfg.name,
            cfg.iface,
            cfg.wait_for_finish,
            cfg.max_pkt,
            cfg.max_byt,
            cfg.duration,
            cfg.send_interval,
            thread_cnt,
            cfg.protocol.into(),
            cfg.payload.try_into().ok(),
        )
    }
}
