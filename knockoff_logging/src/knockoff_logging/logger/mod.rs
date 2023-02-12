use std::future::Future;
use crate::knockoff_logging::log_format::{LogData, LogFormatter};
use crate::knockoff_logging::log_level::LogLevel;

pub trait Logger<T: LogData> {
    type LogFormatterType: LogFormatter<T>;
    type LoggerArgsType: LoggerArgs;

    fn new(log_args: Self::LoggerArgsType) -> Self where Self: Sized;

    fn log(&self, log_level: LogLevel, to_log_message: &str, to_log_trace_id: &str) {
        let formatted = Self::LogFormatterType::format_log(log_level, to_log_message, to_log_trace_id);
        self.write_log(formatted.as_str());
    }

    fn new_from_file() -> Option<Self> where Self: Sized;

    fn log_data(&self, log_level: LogLevel, to_log_data: T);
    fn write_log(&self, log_data: &str);
}

pub trait AsyncLogger<T: LogData>: Logger<T> {
    fn write_log_async(&self, log_data: &str);
    async fn join_log(&self);
}

pub trait LoggerArgs {
}