use std::cell::RefCell;
use std::fmt::Error;
use std::fs::{File, OpenOptions};
use std::future::Future;
use std::io::{Seek, SeekFrom, Write};
use std::ops::DerefMut;
use std::os::fd::AsFd;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use crate::knockoff_logging::log_format::LogFormatter;
use crate::knockoff_logging::log_level::LogLevel;
use crate::knockoff_logging::logger::{Logger, LoggerArgs};
use crate::knockoff_logging::standard_formatter::{StandardLogData, StandardLogFormatter};

pub struct PrintLogger;

pub struct PrintLoggerArgs;

impl LoggerArgs for PrintLoggerArgs {}

impl <'a> Logger<StandardLogData<'a>> for PrintLogger {

    type LogFormatterType = StandardLogFormatter;
    type LoggerArgsType = PrintLoggerArgs;

    fn new(log_args: PrintLoggerArgs) -> Self {
        PrintLogger {}
    }

    fn new_from_file() -> Option<Self> where Self: Sized {
        todo!()
    }

    fn log_data(&self, log_level: LogLevel, to_log_data: StandardLogData<'a>) {
        todo!()
    }

    fn write_log(&self, log_data: &str) {
        println!("{}", log_data);
    }

}
