use anyhow::{Result, anyhow};

use crate::config::base::Config;

impl Config {
    /// Loads configuration from a JSON file on disk.
    ///
    /// # Arguments
    /// * `path` - The path to the configuration file.
    ///
    /// # Returns
    /// * `Result<Self>` - The loaded configuration or an error if loading fails.
    pub fn load_from_file(path: &str) -> Result<Self> {
        let contents = std::fs::read_to_string(path)
            .map_err(|e| anyhow!("Failed to read config file: {}", e))?;

        let cfg: Self = serde_json::from_str(&contents)
            .map_err(|e| anyhow!("Failed to parse config file: {}", e))?;

        Ok(cfg)
    }

    /// Saves the configuration to a JSON file on disk.
    ///
    /// # Arguments
    /// * `path` - The path to the configuration file.
    ///
    /// # Returns
    /// * `Result<()>` - An empty result or an error if saving fails.
    #[allow(dead_code)]
    pub fn save_to_disk(&self, path: &str) -> Result<()> {
        let contents = serde_json::to_string_pretty(self)
            .map_err(|e| anyhow!("Failed to serialize config: {}", e))?;

        std::fs::write(path, contents)
            .map_err(|e| anyhow!("Failed to write config file: {}", e))?;

        Ok(())
    }
}
