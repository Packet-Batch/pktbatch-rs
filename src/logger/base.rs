use serde::{Deserialize, Serialize};

use crate::logger::level::LogLevel;

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct LoggerBase {
    pub log_level: LogLevel,
    pub log_path: Option<String>,

    pub log_path_is_file: bool,

    pub log_date_format_file: Option<String>,
    pub log_date_format_line: Option<String>,

    pub is_watching: bool, // internal state for watcher mode
}

pub type Logger = LoggerBase;

impl LoggerBase {
    pub fn new(
        log_level: LogLevel,
        log_path: Option<String>,
        log_path_is_file: bool,
        log_date_format_file: Option<String>,
        log_date_format_line: Option<String>,
        is_watching: bool,
    ) -> Self {
        Self {
            log_level,
            log_path,
            log_path_is_file,
            log_date_format_file,
            log_date_format_line,
            is_watching,
        }
    }
}
