#[derive(Clone)]
pub struct AfXdpOpts {
    pub queue_id: Option<u16>,
    pub need_wakeup: bool,
    pub shared_umem: bool,
    pub batch_size: u32,
    pub zero_copy: bool,
}

impl Default for AfXdpOpts {
    fn default() -> Self {
        Self {
            queue_id: None,
            need_wakeup: false,
            shared_umem: false,
            batch_size: 64,
            zero_copy: false, // true is best for performance, but it requires a supported driver and kernel version, so we default to false for better compatibility
        }
    }
}

impl AfXdpOpts {
    pub fn new(
        queue_id: Option<u16>,
        need_wakeup: bool,
        shared_umem: bool,
        batch_size: u32,
        zero_copy: bool,
    ) -> Self {
        Self {
            queue_id,
            need_wakeup,
            shared_umem,
            batch_size,
            zero_copy,
        }
    }
}
