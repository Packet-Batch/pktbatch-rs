use std::{fs, io::Write, path::PathBuf};

use anyhow::{Result, anyhow};

use crate::logger::{base::LoggerBase, level::LogLevel};

impl LoggerBase {
    pub fn log_msg(&self, req_level: LogLevel, msg: &str) -> Result<()> {
        // Make sure we have the required log level.
        if req_level < self.log_level {
            return Ok(());
        }

        // Construct line.
        let mut line = String::new();

        line.push_str(format!("[{}] {}", req_level, msg).as_str());

        // Print basic log line to console.
        println!("{}", line);

        // If we don't have a log path, we can just return here.
        if self.log_path.is_none() {
            return Ok(());
        }

        // Retrieve current time for timestamp formation.
        let now = chrono::Local::now();

        // Retrieve timestamp date formats.
        let ts_file = match self.log_date_format_file {
            Some(ref fmt) => now.format(fmt).to_string(),
            None => now.format("%Y-%m-%d").to_string(),
        };

        let ts_line = match self.log_date_format_line {
            Some(ref fmt) => now.format(fmt).to_string(),
            None => now.format("%Y-%m-%d %H:%M:%S").to_string(),
        };

        // Prepend timestamp to line.
        line = format!("[{}] {}", ts_line, line);

        // Determine logging path based off of single/directory settings.
        let log_path = {
            if self.log_path_is_file {
                // If we have a single file, just return.
                self.log_path.as_ref().unwrap().clone()
            } else {
                // Treat log path as directory and append timestamped file name.
                let mut dir = PathBuf::from(self.log_path.as_ref().unwrap());

                dir.push(format!("{}.log", ts_file));

                dir.to_string_lossy().to_string()
            }
        };

        // Attempt to create file/directoy if it doesn't exist and then attempt to write log line.
        fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)
            .map_err(|e| anyhow!("Failed to open log file: {}", e))?
            .write_all(line.as_bytes())
            .map_err(|e| anyhow!("Failed to write log: {}", e))
    }
}
