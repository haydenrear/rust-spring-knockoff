#[macro_export]
macro_rules! initialize_logger {
    () => {
        lazy_static!{
            pub static ref text_file_logger: TextFileLogger = TextFileLogger::new_from_file().unwrap();
        }
    }
}

#[macro_export]
macro_rules! log {
    ($log_level:expr, $message:expr, $trace_id:expr) => {
        text_file_logger.log($log_level, $message, $trace_id)
    }
}

#[macro_export]
macro_rules! log_info {
    ($message:expr, $trace_id:expr) => {
        text_file_logger.log(LogLevel::Info, $message, $trace_id)
    }
}
