use anyhow::{Context, Result, anyhow};
use std::{io::Write, num::NonZeroU32};
use xsk_rs::{
    CompQueue, FrameDesc, Socket, TxQueue, Umem,
    config::{
        BindFlags, FrameSize, Interface, LibxdpFlags, QueueSize, SocketConfig, UmemConfig, XdpFlags,
    },
};

use crate::tech::afxdp::opt::AfXdpOpts;

pub struct XskTxSocket {
    pub umem: Umem,
    pub cq: CompQueue,
    pub tx_q: TxQueue,
    pub descs: Vec<FrameDesc>,
    pub batch_size: usize,
    pub outstanding_tx: u32,
}

pub struct XskTxConfig {
    pub if_name: String,
    pub queue_id: u16,
    pub tx_q_size: u32,
    pub cq_size: u32,
    pub frame_size: u32,
    pub frame_count: u32,
    pub batch_size: usize,
    pub need_wakeup: bool,
    pub zero_copy: bool,
    pub shared_umem: bool,
}

impl From<AfXdpOpts> for XskTxConfig {
    fn from(opts: AfXdpOpts) -> Self {
        Self {
            if_name: String::new(), // must be set by caller
            queue_id: opts.queue_id.unwrap_or(0),
            tx_q_size: 2048,
            cq_size: 2048,
            frame_size: 2048,
            frame_count: 4096, // enough frames for 2 batches
            batch_size: opts.batch_size as usize,
            need_wakeup: opts.need_wakeup,
            zero_copy: opts.zero_copy,
            shared_umem: opts.shared_umem,
        }
    }
}

/// Holds a UMEM that can optionally be shared across multiple sockets.
pub struct XskUmem {
    pub umem: Umem,
    pub descs: Vec<FrameDesc>,
}

impl XskUmem {
    pub fn new(cfg: &XskTxConfig) -> Result<Self> {
        let frame_size: FrameSize = cfg.frame_size.try_into().context("invalid frame size")?;
        let cq_size: QueueSize = cfg.cq_size.try_into().context("invalid cq size")?;

        let umem_config = UmemConfig::builder()
            .frame_size(frame_size)
            .comp_queue_size(cq_size)
            .fill_queue_size(cfg.cq_size.try_into()?)
            .build()
            .context("failed to build umem config")?;

        let frame_count =
            NonZeroU32::new(cfg.frame_count).context("frame count must be non-zero")?;

        let (umem, descs) =
            Umem::new(umem_config, frame_count, cfg.zero_copy).context("failed to create UMEM")?;

        Ok(Self { umem, descs })
    }
}

impl XskTxSocket {
    /// Create a socket with its own dedicated UMEM.
    pub fn new(cfg: XskTxConfig, shared_umem: Option<&XskUmem>) -> Result<Self> {
        let owned_umem;

        let umem = match shared_umem {
            Some(shared) => shared,
            None => {
                owned_umem = XskUmem::new(&cfg)
                    .map_err(|e| anyhow!("failed to create UMEM for socket: {}", e))?;
                &owned_umem
            }
        };

        let bind_flags = Self::build_bind_flags(&cfg);
        let libxdp_flags = Self::build_libxdp_flags();
        let xdp_flags = Self::build_xdp_flags(&cfg);

        let sock_cfg = SocketConfig::builder()
            .tx_queue_size(cfg.tx_q_size.try_into().context("invalid tx queue size")?)
            .bind_flags(bind_flags)
            .libxdp_flags(libxdp_flags) // Prevent BPF XDP program from loading since we're only using TX.
            .xdp_flags(xdp_flags)
            .build();

        let if_name: Interface = cfg.if_name.parse().context("invalid interface name")?;

        let (tx_q, _rx_q, fq_and_cq) = unsafe {
            Socket::new(sock_cfg, &umem.umem, &if_name, cfg.queue_id as u32)
                .context("failed to create AF_XDP socket")?
        };

        let (_fq, cq) =
            fq_and_cq.context("failed to get fill/comp queues for shared umem socket")?;

        Ok(Self {
            umem: umem.umem.clone(),
            cq,
            tx_q,
            descs: umem.descs.clone(),
            batch_size: cfg.batch_size,
            outstanding_tx: 0,
        })
    }

    fn build_bind_flags(cfg: &XskTxConfig) -> BindFlags {
        let mut flags = BindFlags::empty();

        if cfg.zero_copy {
            flags |= BindFlags::XDP_ZEROCOPY;
        } else {
            flags |= BindFlags::XDP_COPY;
        }

        flags
    }

    fn build_libxdp_flags() -> LibxdpFlags {
        LibxdpFlags::XSK_LIBXDP_FLAGS_INHIBIT_PROG_LOAD
    }

    fn build_xdp_flags(cfg: &XskTxConfig) -> XdpFlags {
        let mut flags = XdpFlags::empty();

        if cfg.zero_copy {
            flags |= XdpFlags::XDP_FLAGS_DRV_MODE;
        } else {
            flags |= XdpFlags::XDP_FLAGS_SKB_MODE;
        }

        flags
    }

    #[inline(always)]
    pub fn send(&mut self, pkt: &[u8]) -> Result<()> {
        self.descs[0].reset_lengths();
        unsafe {
            self.umem
                .data_mut(&mut self.descs[0])
                .cursor()
                .write_all(pkt)?
        }

        loop {
            if unsafe { self.tx_q.produce_and_wakeup(&self.descs[..1]) }.unwrap_or(0) > 0 {
                break;
            }
            unsafe { self.cq.consume(&mut self.descs[..]) };
        }

        self.outstanding_tx += 1;

        if self.outstanding_tx >= self.batch_size as u32 {
            self.complete_tx()?;
        }

        Ok(())
    }

    fn complete_tx(&mut self) -> Result<()> {
        self.tx_q.wakeup().ok();

        // Keep draining until outstanding is under control
        while self.outstanding_tx > 0 {
            let n = unsafe {
                self.cq
                    .consume(&mut self.descs[self.batch_size..self.batch_size * 2])
            };

            if n == 0 {
                break;
            }

            self.outstanding_tx = self.outstanding_tx.saturating_sub(n as u32);
        }

        Ok(())
    }
}
