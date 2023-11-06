use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::env;
use std::fmt::{Debug, Error, Formatter};
use std::fs::{File, OpenOptions};
use std::future::Future;
use std::io::{Seek, SeekFrom, Write};
use std::ops::DerefMut;
use std::os::fd::AsFd;
use std::path::{Path, PathBuf};
use std::ptr::write;
use std::sync::{Arc, Mutex};
use lazy_static::lazy_static;
use crate::log_format::{LogData, LogFormatter};
use crate::log_level::LogLevel;
use crate::logger::{AsyncLogger, Logger, LoggerArgs};
use crate::standard_formatter::{StandardLogData, StandardLogFormatter};

pub struct TextFileLoggerImpl {
    pub(crate) text_file: File,

}

impl Debug for TextFileLoggerImpl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("")
    }
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

pub trait TextFileLogger<T: LogData>: Logger<T> {
    fn new_from_file() -> Option<Self> where Self: Sized;
    fn new_from_file_dir(logging_dir: &str) -> Option<Self> where Self: Sized;
}

lazy_static! {
    pub static ref static_: Mutex<String> = {
        Mutex::new(String::from(""))
    };
}

impl Logger<StandardLogData> for TextFileLoggerImpl {
    type LogFormatterType = StandardLogFormatter;
    type LoggerArgsType = TextFileLoggerArgs;

    fn new(log_args: TextFileLoggerArgs) -> Self {
        let out = static_.lock().as_mut().unwrap();
        TextFileLoggerImpl {
            text_file: log_args.file,
        }
    }


    fn log_data(&self, log_level: LogLevel, to_log_data: StandardLogData) {
        todo!()
    }

    fn write_log(&mut self, log_data: String) {
        let _ = self.text_file.write(log_data.as_bytes()).map_err(|e| {
            println!("Failed to log {}", e.to_string().as_str());
        });
    }
}

impl TextFileLogger<StandardLogData> for TextFileLoggerImpl {

    fn new_from_file() -> Option<Self> {
        let logging_file_result = env::var("LOGGING_DIR").ok();

        print!("Creating logger. {}", logging_file_result.clone().as_ref().unwrap().as_str().clone());

        logging_file_result.and_then(|file_path| {
            Self::create_logger(&file_path)
        })
    }

    fn new_from_file_dir(logging_dir: &str) -> Option<Self> {
        Self::create_logger(logging_dir)
    }

}

impl TextFileLoggerImpl {
    fn create_logger(file_path: &str) -> Option<TextFileLoggerImpl> {
        let file_path = Path::new(&file_path);
        if !file_path.exists() {
            let created_file = File::create(file_path);
            if created_file.is_err() {
                panic!("Failed to create file: {}!", file_path.to_str().unwrap());
            }
        }

        File::options()
            .append(true)
            .open(file_path)
            .ok()
            .and_then(|mut file| Some(TextFileLoggerArgs { file }))
            .map(|logger_args| TextFileLoggerImpl { text_file: logger_args.file}  )
            .or_else(|| panic!("Failed to create logger!"))
    }
}