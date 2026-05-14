use anyhow::{Context as AnyhowCtx, Result};
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};

use crate::watcher::base::Watcher;

impl Watcher {
    pub async fn run(&mut self) -> Result<()> {
        enable_raw_mode()?;

        let mut stdout = Watcher::get_stdout();
        execute!(stdout, EnterAlternateScreen, crossterm::cursor::Hide)?;

        self.terminal.clear()?;

        let result = self
            .interface_start()
            .await
            .context("Failed to start interface watcher");

        disable_raw_mode()?;

        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            crossterm::cursor::Show
        )?;

        result
    }
}
