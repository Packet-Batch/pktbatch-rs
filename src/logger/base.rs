use serde::{Deserialize, Serialize};

use crate::logger::level::LogLevel;

#[derive(Clone, Serialize, Deserialize)]
pub struct LoggerBase {
    pub log_level: LogLevel,
    pub log_path: Option<String>,

    pub log_path_is_file: bool,
    pub log_date_format: String,
}

pub type Logger = LoggerBase;

impl LoggerBase {
    pub fn new(
        log_level: LogLevel,
        log_path: Option<String>,
        log_path_is_file: bool,
        log_date_format: String,
    ) -> Self {
        Self {
            log_level,
            log_path,
            log_path_is_file,
            log_date_format,
        }
    }
}

impl Default for LoggerBase {
    fn default() -> Self {
        Self {
            log_level: LogLevel::Info,
            log_path: None,
            log_path_is_file: false,
            log_date_format: "%Y-%m-%d %H:%M:%S".to_string(),
        }
    }
}
