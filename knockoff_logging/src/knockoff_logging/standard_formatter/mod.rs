use std::fmt::Formatter;
use crate::knockoff_logging::log_format::{LogData, LogFormatter};
use crate::knockoff_logging::log_level::{LogLevel, LogLevels};

pub struct StandardLogFormatter;

impl LogData for StandardLogData {

}
pub struct StandardLogData {
    message: String,
    trace_id: String,
    date: u64,
    log_level: LogLevel
}

impl LogFormatter<StandardLogData> for StandardLogFormatter {

    fn format_log<'b>(log_level: LogLevel, text: String, id: String) -> String {
        let mut formatted = "".to_string();
        formatted = formatted + "log level: ";
        formatted = formatted + LogLevels[&log_level];
        formatted = formatted + "\n";
        formatted = formatted + "message: ";
        formatted = formatted + text.as_str();
        formatted = formatted + "\n";
        formatted = formatted + "id: ";
        formatted = formatted + id.as_str();
        formatted = formatted + "\n";
        formatted
    }

    fn format_data<'b>(log_data: StandardLogData) -> String {
        todo!()
    }
}