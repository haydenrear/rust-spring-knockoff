use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::env;
use std::fmt::Error;
use std::fs::{File, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::ops::DerefMut;
use std::os::fd::AsFd;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use crate::knockoff_logging::log_format::LogFormatter;
use crate::knockoff_logging::log_level::LogLevel;
use crate::knockoff_logging::logger::{Logger, LoggerArgs};
use crate::knockoff_logging::standard_formatter::{StandardLogData, StandardLogFormatter};

pub struct TextFileLogger {
    text_file: Arc<Mutex<File>>
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

impl TextFileLogger {
    pub fn new_from_file() -> Option<Self> {
        let logging_file_result = env::var("LOGGING_DIR").ok();

        logging_file_result.and_then(|file_path| {
            let file_path = Path::new(&file_path);
            if !file_path.exists() {
                let created_file = File::create(file_path);
                if created_file.is_err() {
                    return None;
                }
            }

            File::create(file_path)
                .ok()
                .and_then(|mut file| Some(TextFileLoggerArgs {file}))
                .map(|logger_args| TextFileLogger { text_file: Arc::new(Mutex::new(logger_args.file)) })
        })

    }
}

impl <'a> Logger<StandardLogData<'a>> for TextFileLogger {
    type LogFormatterType = StandardLogFormatter;
    type LoggerArgsType = TextFileLoggerArgs;

    fn new(log_args: TextFileLoggerArgs) -> Self {
        TextFileLogger {
            text_file: Arc::new(Mutex::new(log_args.file))
        }
    }

    fn log_data(&self, log_level: LogLevel, to_log_data: StandardLogData<'a>) {
        todo!()
    }

    fn write_log(&self, log_data: &str) {
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