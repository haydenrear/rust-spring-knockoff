use std::fmt::Formatter;
use crate::knockoff_logging::log_format::{LogData, LogFormatter};
use crate::knockoff_logging::log_level::{LogLevel, LogLevels};

pub struct StandardLogFormatter;

impl <'a> LogData for StandardLogData<'a> {

}
pub struct StandardLogData<'a> {
    message: &'a str,
    trace_id: &'a str,
    date: u64,
    log_level: LogLevel
}

impl <'a> LogFormatter<StandardLogData<'a>> for StandardLogFormatter {

    fn format_log<'b>(log_level: LogLevel, text: &'b str, id: &'b str) -> String {
        let mut formatted = "".to_string();
        formatted = formatted + "log level: ";
        formatted = formatted + LogLevels[&log_level];
        formatted = formatted + "\n";
        formatted = formatted + "message: ";
        formatted = formatted + text;
        formatted = formatted + "\n";
        formatted = formatted + "id: ";
        formatted = formatted + id;
        formatted = formatted + "\n";
        formatted
    }

    fn format_data<'b>(log_data: StandardLogData) -> String {
        todo!()
    }
}