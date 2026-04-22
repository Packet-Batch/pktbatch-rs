use anyhow::Result;
use anyhow::anyhow;

use crate::config::batch::data::payload::PayloadOpts as PayloadOptsCfg;
use crate::util::rand_fast::pcg32_fast;
use crate::util::rand_num;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Payload {
    pub len_min: Option<u16>,
    pub len_max: Option<u16>,

    pub is_static: bool,
    pub is_file: bool,
    pub is_string: bool,

    pub exact: Option<String>,
}

impl From<PayloadOptsCfg> for Payload {
    fn from(cfg: PayloadOptsCfg) -> Self {
        Payload {
            len_min: cfg.len_min,
            len_max: cfg.len_max,
            is_static: cfg.is_static,
            is_file: cfg.is_file,
            is_string: cfg.is_string,
            exact: cfg.exact,
        }
    }
}

impl Payload {
    /// Generates payload based off of configured options. Also determines whether the payload is static or not.
    ///
    /// # Arguments
    /// * `buf` - The buffer to write the generated payload into.
    /// * `seed` - The seed used to generate random payload. This is used to ensure that the same random payload is generated across different threads if the same seed is used.
    ///
    /// # Returns
    /// A `Result` containing an `Option` with a tuple of the payload length and a boolean indicating whether the payload is static (true) or random (false). If no payload is generated, returns `Ok(None)`. If there is an error during payload generation, returns an `anyhow::Error`.
    #[inline(always)]
    pub fn gen_payload(
        &self,
        buf: &mut [u8],
        seed: &mut u64,
        proto_len: usize,
    ) -> Result<Option<(u16, bool)>> {
        // Check for exact first.
        if let Some(exact) = &self.exact {
            // Check if we should read the payload data from a file or use the string directly.
            let exact = if self.is_file {
                std::fs::read_to_string(exact)
                    .map_err(|e| anyhow!("Failed to read payload from file: {}", e))?
            } else {
                exact.clone()
            };

            let bytes = {
                // If the payload is a string, we can just use the bytes of the string.
                if self.is_string {
                    exact.as_bytes().to_vec()
                } else {
                    // Otherwise, we'll try to parse as hex.
                    exact
                        .split_whitespace()
                        .map(|byt_str| {
                            u8::from_str_radix(byt_str, 16).map_err(|e| {
                                anyhow!("Failed to parse byte '{}' as hex: {}", byt_str, e)
                            })
                        })
                        .collect::<Result<Vec<u8>, _>>()
                        .map_err(|e| anyhow!("Failed to parse exact payload as hex: {}", e))?
                }
            };

            let len = bytes.len().min(buf.len());
            buf[..len].copy_from_slice(&bytes[..len]);

            return Ok(Some((len as u16, true)));
        }

        let min_len = self.len_min.unwrap_or(0) as usize;
        let max_len = self.len_max.unwrap_or(0) as usize;

        if min_len < 1 && max_len < 1 {
            return Ok(None);
        }

        // Now we can generate random payload if needed.
        let len = {
            if max_len < min_len {
                return Err(anyhow!("len_max must be greater than or equal to len_min"));
            }

            rand_num(seed, min_len as u64, max_len as u64) as usize
        };

        // Instead of RNG, let's use our fast random generator.
        // We do this by using pcg32_fast() which generates 4 bytes. So split the buffer into 4 byte chunks to fill and ensure we account for the remainder of the bytes.
        let chunks = len / 4;
        let remainder = len % 4;

        for i in 0..chunks {
            let rand = pcg32_fast(seed).to_le_bytes();

            let boundry = (i * 4 + 4).min(len);

            buf[i * 4..boundry].copy_from_slice(&rand);
        }

        // Ensure we aren't forgetting the remaining bytes.
        if remainder > 0 {
            let rand = pcg32_fast(seed).to_le_bytes();

            buf[chunks * 4..chunks * 4 + remainder].copy_from_slice(&rand[..remainder]);
        }

        println!("EEE: {} == {}", min_len, max_len);

        Ok(Some((len as u16, self.is_static && min_len == max_len)))
    }
}
