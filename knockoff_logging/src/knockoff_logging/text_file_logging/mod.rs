use std::cell::RefCell;
use std::fmt::Error;
use std::fs::{File, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::ops::DerefMut;
use std::os::fd::AsFd;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use crate::knockoff_logging::log_format::LogFormatter;
use crate::knockoff_logging::log_level::LogLevel;
use crate::knockoff_logging::logger::{Logger, LoggerArgs};
use crate::knockoff_logging::standard_formatter::{StandardLogData, StandardLogFormatter};

pub struct TextFileLogger {
    text_file: RefCell<File>
}

pub struct TextFileLoggerArgs {
    pub file: File
}

impl TextFileLoggerArgs {
    pub fn new(mut file_path: &PathBuf) -> Option<Self> {
        if !file_path.exists() {
            let created_file = File::create(file_path);
            if created_file.is_err() {
                return None;
            }
        }
        File::create(file_path)
            .ok()
            .and_then(|mut file| Some(Self {file}))
    }
}

impl LoggerArgs for TextFileLoggerArgs {}

impl <'a> Logger<StandardLogData<'a>> for TextFileLogger {
    type LogFormatterType = StandardLogFormatter;
    type LoggerArgsType = TextFileLoggerArgs;

    fn new(log_args: TextFileLoggerArgs) -> Self {
        TextFileLogger {
            text_file: RefCell::new(log_args.file)
        }
    }

    fn log_data(&self, log_level: LogLevel, to_log_data: StandardLogData<'a>) {
        todo!()
    }

    fn write_log(&self, log_data: &str) {
        let mut mut_file = self.text_file.borrow_mut();
        mut_file.seek(SeekFrom::End(0)).ok().and_then(|seeked| {
            mut_file.write(log_data.as_bytes()).ok()
        })
            .or_else(|| {
                println!("Failed to log." );
                None
            });
    }
}