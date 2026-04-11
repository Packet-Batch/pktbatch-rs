use crate::logger::{base::LoggerBase, level::LogLevel};;

impl LoggerBase {
    pub fn log_msg(&self, req_level: LogLevel, msg: &str) {
        if req_level < self.log_level {
            return;
        }

        let now = chrono::Local::now();
        let timestamp = now.format(&self.log_date_format).to_string();

        let log_line = format!("[{}] [{}] {}", req_level as u8, timestamp, msg);

        if let Some(ref path) = self.log_path {
            if self.log_path_is_file {
                std::fs::write(path, log_line + "\n").expect("Failed to write log");
            } else {
                std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)
                    .expect("Failed to open log file")
                    .write_all(log_line.as_bytes())
                    .expect("Failed to write log");
            }
        } else {
            println!("{}", log_line);
        }
    }
}
