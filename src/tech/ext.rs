use anyhow::Result;

use async_trait::async_trait;

use crate::context::Context;

#[async_trait]
pub trait TechExt {
    type Tech;
    type Opts;

    type TechDataInit;
    type TechDataThread;

    /// Creates a new instance of the packet tech with the given options.
    ///
    /// # Arguments
    /// * `opts` - The options for configuring the packet tech.
    ///
    /// # Returns
    /// * `Self` - A new instance of the packet tech.
    fn new(opts: Self::Opts) -> Self;

    /// Retrieves a reference to the underlying packet tech.
    ///
    /// # Returns
    /// * `&Self::Tech` - A reference to the underlying packet tech.
    fn get(&self) -> &Self::Tech;

    /// Retrieves a mutable reference to the underlying packet tech.
    ///
    /// # Returns
    /// * `&mut Self::Tech` - A mutable reference to the underlying packet tech.
    fn get_mut(&mut self) -> &mut Self::Tech;

    /// Initializes the packet tech. This is where setup takes place.
    ///
    /// # Arguments
    /// * `ctx` - The context of the application, which contains shared data and resources.
    /// * `iface_fb` - An optional interface name to bind to, which may be required for certain packet techs.
    ///
    /// # Returns
    /// * `Result<()>` - Returns `Ok(())` if initialization is successful, or an error if it fails.
    async fn init(
        &mut self,
        ctx: Context,
        iface_fb: Option<String>,
    ) -> Result<Option<Self::TechDataInit>>;

    /// Setups info for the specific thread.
    ///
    /// # Arguments
    /// * `ctx` - The context of the application, which contains shared data and resources.
    /// * `thread_id` - The ID of the thread for which to setup info.
    /// * `iface_fb` - An optional interface name to bind to, which may be required for certain packet techs.
    ///
    /// # Returns
    /// A `Result` with the thread-specific data if successful, or an `anyhow::Error` if setup fails.
    fn init_thread(
        &mut self,
        ctx: Context,
        thread_id: u16,
        iface_fb: Option<String>,
    ) -> Result<Option<Self::TechDataThread>>;

    /// Sends a packet.
    ///
    /// # Arguments
    /// * `pkt` - The packet data to be sent.
    /// * `data_init` - Additional data specific to the packet tech, which may be required for sending the packet.
    /// * `data_thread` - Thread-specific data for the packet tech, which may be required for sending the packet.
    ///
    /// # Returns
    /// True if the packet was sent successfully, or false if it failed.
    fn pkt_send(&mut self, pkt: &[u8], data_thread: Option<&mut Self::TechDataThread>) -> bool;
}
