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
            cq_size: opts.batch_size,
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
        })
    }

    fn build_bind_flags(cfg: &XskTxConfig) -> BindFlags {
        let mut flags = BindFlags::empty();

        if cfg.need_wakeup {
            flags |= BindFlags::XDP_USE_NEED_WAKEUP;
        }

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
    fn submit_and_drain(&mut self, count: usize) -> Result<()> {
        let submitted = unsafe { self.tx_q.produce(&self.descs[..count]) };

        if submitted == 0 {
            anyhow::bail!("tx queue failed to accept {} frame(s)", count);
        }

        let mut completed = vec![self.descs[0]; count];
        let mut remaining = count;

        while remaining > 0 {
            if self.tx_q.needs_wakeup() {
                self.tx_q.wakeup().context("tx wakeup failed")?;
            }

            let n = unsafe { self.cq.consume(&mut completed[..remaining]) };
            remaining = remaining.saturating_sub(n);
        }

        Ok(())
    }

    /// Send a single packet.
    #[inline(always)]
    pub fn send(&mut self, pkt: &[u8]) -> Result<()> {
        let desc = self
            .descs
            .first_mut()
            .context("no frame descriptors available")?;

        unsafe {
            self.umem.data_mut(desc).cursor().write_all(pkt)?;
        }

        self.submit_and_drain(1)
    }

    /// Send a batch of packets, chunked by `self.batch_size`.
    #[inline(always)]
    pub fn send_batch(&mut self, pkts: &[&[u8]]) -> Result<()> {
        for chunk in pkts.chunks(self.batch_size) {
            let count = chunk.len();

            for (i, pkt) in chunk.iter().enumerate() {
                let desc = self
                    .descs
                    .get_mut(i)
                    .context("not enough frame descriptors")?;
                unsafe {
                    self.umem.data_mut(desc).cursor().write_all(pkt)?;
                }
            }

            self.submit_and_drain(count)?;
        }

        Ok(())
    }

    /// Send the same packet repeated `batch_size` times.
    #[inline(always)]
    pub fn send_repeated(&mut self, pkt: &[u8]) -> Result<()> {
        let count = self.batch_size;

        for i in 0..count {
            let desc = match self.descs.get_mut(i) {
                Some(d) => d,
                None => {
                    return Err(anyhow!(
                        "not enough frame descriptors for repeated send: needed {}, have {}",
                        count,
                        self.descs.len()
                    ));
                }
            };

            unsafe {
                self.umem.data_mut(desc).cursor().write_all(pkt)?;
            }
        }

        match self.submit_and_drain(count) {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow!("failed to send repeated packet: {}", e)),
        }
    }
}
