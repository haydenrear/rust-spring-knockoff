use std::cell::RefCell;
use std::fmt::Error;
use std::fs::{File, OpenOptions};
use std::future::Future;
use std::io::{Seek, SeekFrom, Write};
use std::ops::DerefMut;
use std::os::fd::AsFd;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use crate::log_format::LogFormatter;
use crate::log_level::LogLevel;
use crate::logger::{Logger, LoggerArgs};
use crate::standard_formatter::{StandardLogData, StandardLogFormatter};

pub struct PrintLogger;

pub struct PrintLoggerArgs;

impl LoggerArgs for PrintLoggerArgs {}

impl Logger<StandardLogData> for PrintLogger {

    type LogFormatterType = StandardLogFormatter;
    type LoggerArgsType = PrintLoggerArgs;

    fn new(log_args: PrintLoggerArgs) -> Self {
        PrintLogger {}
    }

    fn log_data(&self, log_level: LogLevel, to_log_data: StandardLogData) {
        todo!()
    }

    fn write_log(&mut self, log_data: String) {
        println!("{}", log_data);
    }

}
