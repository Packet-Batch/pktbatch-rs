use std::sync::Arc;

use tokio::sync::RwLock;

use crate::{config::base::Config, logger::base::Logger};

pub struct ContextData {
    pub cfg: RwLock<Config>,
    pub logger: RwLock<Logger>,
}

pub type Context = Arc<ContextData>;

impl ContextData {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            cfg: RwLock::new(Config::default()),
            logger: RwLock::new(Logger::default()),
        })
    }
}