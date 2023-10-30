use std::future::Future;
use crate::log_format::{LogData, LogFormatter};
use crate::log_level::LogLevel;

pub trait Logger<T: LogData> {
    type LogFormatterType: LogFormatter<T>;
    type LoggerArgsType: LoggerArgs;

    fn new(log_args: Self::LoggerArgsType) -> Self where Self: Sized;

    fn log(&mut self, log_level: LogLevel, to_log_message: String, to_log_trace_id: String) {
        let formatted = Self::LogFormatterType::format_log(log_level, to_log_message, to_log_trace_id);
        self.write_log(formatted);
    }


    fn log_data(&self, log_level: LogLevel, to_log_data: T);

    fn write_log(&mut self, log_data: String);

}

pub trait AsyncLogger<T: LogData>: Logger<T> {
    fn write_log_async(&self, log_data: &str);
    async fn join_log(&self);
}

pub trait LoggerArgs {
}