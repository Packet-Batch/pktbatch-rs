use serde::{Deserialize, Serialize};

use crate::logger::level::LogLevel;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Logger {
    pub level: Option<LogLevel>,
    pub path: Option<String>,

    pub path_is_file: bool,

    pub date_format_file: Option<String>,
    pub date_format_line: Option<String>,
}

impl Default for Logger {
    fn default() -> Self {
        Self {
            level: Some(LogLevel::Info),
            path: Some("logs/".to_string()),
            path_is_file: false,
            date_format_file: Some("%Y-%m-%d".to_string()),
            date_format_line: Some("%Y-%m-%d %H:%M:%S".to_string()),
        }
    }
}
