use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use crate::logger::level::LogLevel;

pub type LogBuffer = Arc<Mutex<VecDeque<String>>>;

#[derive(Clone, Default)]
pub struct LoggerBase {
    pub log_level: LogLevel,
    pub log_path: Option<String>,

    pub log_path_is_file: bool,

    pub log_date_format_file: Option<String>,
    pub log_date_format_line: Option<String>,

    pub buffer: Option<LogBuffer>,
}

pub type Logger = LoggerBase;

impl LoggerBase {
    pub fn new(
        log_level: LogLevel,
        log_path: Option<String>,
        log_path_is_file: bool,
        log_date_format_file: Option<String>,
        log_date_format_line: Option<String>,
        buffer: Option<LogBuffer>,
    ) -> Self {
        Self {
            log_level,
            log_path,
            log_path_is_file,
            log_date_format_file,
            log_date_format_line,
            buffer,
        }
    }
}

pub const DEFAULT_BACKLOG_SZ: usize = 200;
