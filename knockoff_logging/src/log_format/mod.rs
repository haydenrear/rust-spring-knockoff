use crate::log_level::LogLevel;

pub trait LogFormatter<T> {
    fn format_log(log_level: LogLevel, text: String, id: String) -> String;
    fn format_data(log_data: T) -> String;
}

pub trait LogData {
}