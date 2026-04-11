use anyhow::Result;

use async_trait::async_trait;

use crate::context::Context;

#[async_trait]
pub trait TechExt {
    /// Initializes the packet tech. This is where setup takes place.
    /// 
    /// # Arguments
    /// * `ctx` - The context of the application, which contains shared data and resources.
    /// 
    /// # Returns
    /// * `Result<()>` - Returns `Ok(())` if initialization is successful, or an error if it fails.
    async fn init(&mut self, ctx: Context) -> Result<()>;

    /// Generates and sends a batch of packets to a single destination.
    /// 
    /// # Arguments
    /// * `ctx` - The context of the application, which contains shared data and resources.
    /// 
    /// # Returns
    /// * `Result<()>` - Returns `Ok(())` if the packet is sent successfully, or an error if it fails.
    async fn batch_start(&mut self, ctx: Context) -> Result<()>;
    async fn batch_end(&mut self, ctx: Context) -> Result<()>;

    async fn pkt_send(&mut self, ctx: Context, pkt: &[u8]) -> Result<()>;
}