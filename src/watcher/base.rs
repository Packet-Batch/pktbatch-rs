use anyhow::{Context as AnyhowCtx, Result};
use std::{collections::VecDeque, io::Stdout};

use crate::{context::Context, logger::base::LogBuffer};
use ratatui::{Terminal, backend::CrosstermBackend};

pub const HISTORY_LEN: usize = 60;

pub struct Watcher {
    pub ctx: Context,
    pub terminal: Terminal<CrosstermBackend<Stdout>>,
    pub logs: Option<LogBuffer>,
    pub history_tot: [VecDeque<f64>; 2],
    pub history_rate: [VecDeque<f64>; 2],
    pub iface_fb: Option<String>,
    pub mode: WatcherMode,

    pub refresh_rate: u64,
}

#[derive(Eq, PartialEq)]
pub enum WatcherMode {
    Total,
    Rate,
}

impl Watcher {
    pub fn new(
        ctx: Context,
        logs: Option<LogBuffer>,
        iface_fb: Option<String>,
        refresh_rate: u64,
    ) -> Result<Self> {
        let stdout = Self::get_stdout();
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend).context("Failed to initialize terminal")?;

        Ok(Self {
            ctx,
            terminal,
            logs,
            history_tot: [VecDeque::new(), VecDeque::new()],
            history_rate: [VecDeque::new(), VecDeque::new()],
            iface_fb,
            mode: WatcherMode::Total,
            refresh_rate,
        })
    }

    pub fn get_stdout() -> Stdout {
        std::io::stdout()
    }
}
