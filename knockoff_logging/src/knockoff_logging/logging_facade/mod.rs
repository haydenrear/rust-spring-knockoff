use crate::initialize_logger;
use crate::knockoff_logging::log_format::{LogData, LogFormatter};
use crate::knockoff_logging::logger::{Logger, LoggerArgs};
use crate::knockoff_logging::standard_formatter::{StandardLogData, StandardLogFormatter};
use crate::knockoff_logging::text_file_logging::TextFileLoggerArgs;
use lazy_static::lazy_static;
use crate::knockoff_logging::text_file_logging::TextFileLoggerImpl;
use std::sync::Arc;
use std::sync::Mutex;
use executors::threadpool_executor::ThreadPoolExecutor;


pub trait LoggingFacade<T: LogData, L: Logger<T, LogFormatterType=Self::LogFormatterType, LoggerArgsType=Self::LoggerArgsType>> {
    type LogFormatterType: LogFormatter<T>;
    type LoggerArgsType: LoggerArgs;
    fn get_logger() -> &'static L;
}
