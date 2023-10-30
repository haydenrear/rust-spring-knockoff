use crate::initialize_logger;
use crate::log_format::{LogData, LogFormatter};
use crate::logger::{Logger, LoggerArgs};
use crate::standard_formatter::{StandardLogData, StandardLogFormatter};
use crate::text_file_logging::TextFileLoggerArgs;
use lazy_static::lazy_static;
use crate::text_file_logging::TextFileLoggerImpl;
use std::sync::{Arc, MutexGuard, PoisonError};
use std::sync::Mutex;


pub trait LoggingFacade<T: LogData, L: Logger<T, LogFormatterType=Self::LogFormatterType, LoggerArgsType=Self::LoggerArgsType>> {
    type LogFormatterType: LogFormatter<T>;
    type LoggerArgsType: LoggerArgs;
    fn get_logger() -> &'static Mutex<L>;
    fn package() -> &'static str;
}
