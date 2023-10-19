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
use executors::Executor;
use executors::threadpool_executor::ThreadPoolExecutor;
use crate::knockoff_logging::log_format::{LogData, LogFormatter};
use crate::knockoff_logging::log_level::LogLevel;
use crate::knockoff_logging::logger::{AsyncLogger, Logger, LoggerArgs};
use crate::knockoff_logging::standard_formatter::{StandardLogData, StandardLogFormatter};

pub struct TextFileLoggerImpl {
    pub(crate) text_file: Arc<Mutex<File>>,
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


impl Logger<StandardLogData> for TextFileLoggerImpl {
    type LogFormatterType = StandardLogFormatter;
    type LoggerArgsType = TextFileLoggerArgs;

    fn new(log_args: TextFileLoggerArgs) -> Self {
        TextFileLoggerImpl {
            text_file: Arc::new(Mutex::new(log_args.file)),
        }
    }


    fn log_data(&self, log_level: LogLevel, to_log_data: StandardLogData) {
        todo!()
    }

    fn write_log(&self, log_data: String) {
        self.text_file.lock().and_then(|mut file| {
            file.seek(SeekFrom::End(0)).ok().and_then(|seeked| {
                file.write(log_data.as_bytes()).ok()
            })
                .or_else(|| {
                    println!("Failed to log." );
                    None
                });
            Ok(())
        }).unwrap();
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

        File::create(file_path)
            .ok()
            .and_then(|mut file| Some(TextFileLoggerArgs { file }))
            .map(|logger_args| TextFileLoggerImpl { text_file: Arc::new(Mutex::new(logger_args.file)) })
            .or_else(|| panic!("Failed to create logger!"))
    }
}