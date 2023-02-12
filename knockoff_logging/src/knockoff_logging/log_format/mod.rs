use crate::knockoff_logging::log_level::LogLevel;

pub trait LogFormatter<T> {
    fn format_log<'a>(log_level: LogLevel, text: &'a str, id: &'a str) -> String;
    fn format_data<'a>(log_data: T) -> String;
}

pub trait LogData {
}