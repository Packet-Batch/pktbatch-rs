use std::sync::{Arc, atomic::AtomicBool};

use tokio::sync::RwLock;

use crate::{
    batch::base::Batch, cli::base::Cli, config::base::Config, logger::base::Logger,
    tech::base::Tech,
};

pub struct ContextData {
    pub cfg: RwLock<Config>,
    pub logger: RwLock<Logger>,
    pub cli: RwLock<Cli>,

    pub tech: RwLock<Tech>,

    pub batch: RwLock<Batch>,

    pub running: Arc<AtomicBool>,
}

pub type Context = Arc<ContextData>;

impl ContextData {
    pub fn new(cfg: Config, logger: Logger, cli: Cli, tech: Tech, batch: Batch) -> Arc<Self> {
        Arc::new(Self {
            cfg: RwLock::new(cfg),
            logger: RwLock::new(logger),
            cli: RwLock::new(cli),
            tech: RwLock::new(tech),
            batch: RwLock::new(batch),
            running: Arc::new(AtomicBool::new(true)),
        })
    }
}
