use anyhow::{Result, anyhow};
use pnet::packet::icmp::{IcmpCode, IcmpType, MutableIcmpPacket, checksum};

use crate::{
    batch::data::{
        ip::IP_HDR_LEN,
        protocol::{Protocol, ProtocolExt},
    },
    config::batch::data::protocol::icmp::IcmpOpts as IcmpOptsCfg,
};

pub const LEN_ICMP_HDR: usize = 8;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IcmpOpts {
    pub icmp_type: u8,
    pub icmp_code: u8,

    pub do_csum: bool,
}

impl From<IcmpOptsCfg> for IcmpOpts {
    fn from(cfg: IcmpOptsCfg) -> Self {
        Self {
            icmp_type: cfg.icmp_type.unwrap_or_default(),
            icmp_code: cfg.icmp_code.unwrap_or_default(),
            do_csum: cfg.do_csum,
        }
    }
}

impl From<IcmpOpts> for IcmpOptsCfg {
    fn from(opts: IcmpOpts) -> Self {
        Self {
            icmp_type: Some(opts.icmp_type),
            icmp_code: Some(opts.icmp_code),
            do_csum: opts.do_csum,
        }
    }
}

impl Default for IcmpOpts {
    fn default() -> Self {
        IcmpOpts {
            icmp_type: 8, // Echo Request
            icmp_code: 0,
            do_csum: true,
        }
    }
}

impl From<IcmpOpts> for Protocol {
    fn from(opts: IcmpOpts) -> Self {
        Protocol::Icmp(opts)
    }
}

impl ProtocolExt for IcmpOpts {
    type Opts = IcmpOpts;
    type State = (); // Not used for ICMP

    #[inline(always)]
    fn get_hdr_len(&self) -> usize {
        LEN_ICMP_HDR
    }

    #[inline(always)]
    fn get_proto_num(&self) -> u8 {
        1
    }

    #[inline(always)]
    fn get_src_port(&self) -> Option<u16> {
        None
    }

    #[inline(always)]
    fn gen_src_port(&self, _buff: &mut [u8], _seed: &mut u64) -> Result<(Option<u16>, bool)> {
        Ok((None, false))
    }

    #[inline(always)]
    fn get_dst_port(&self) -> Option<u16> {
        None
    }

    #[inline(always)]
    fn gen_dst_port(&self, _buff: &mut [u8], _seed: &mut u64) -> Result<(Option<u16>, bool)> {
        Ok((None, false))
    }

    #[inline(always)]
    fn gen_checksum(&self, buff: &mut [u8]) -> Result<()> {
        if !self.do_csum {
            return Ok(());
        }

        let mut icmph = match MutableIcmpPacket::new(buff[IP_HDR_LEN..].as_mut()) {
            Some(p) => p,
            None => {
                return Err(anyhow!(
                    "Failed to create mutable ICMP packet for checksum calculation"
                ));
            }
        };

        icmph.set_checksum(
            checksum(&icmph.to_immutable())
                .try_into()
                .map_err(|e| anyhow!("Failed to convert checksum to u16: {}", e))?,
        );

        Ok(())
    }

    #[inline(always)]
    fn fill_init(&self, buff: &mut [u8], _seed: &mut u64) -> Result<(bool, bool)> {
        let mut icmph = match MutableIcmpPacket::new(buff) {
            Some(p) => p,
            None => {
                return Err(anyhow!(
                    "Failed to create mutable ICMP packet for filling fields"
                ));
            }
        };

        icmph.set_icmp_type(IcmpType(self.icmp_type));
        icmph.set_icmp_code(IcmpCode(self.icmp_code));

        Ok((true, true))
    }

    #[inline(always)]
    fn fill(&self, _buff: &mut [u8], _flags: u32, _seed: &mut u64) -> Result<()> {
        Ok(())
    }

    #[inline(always)]
    fn set_total_len(&self, _buff: &mut [u8], _new_len: u16) -> Result<()> {
        Ok(())
    }
}
