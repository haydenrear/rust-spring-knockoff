#[macro_export]
macro_rules! initialize_logger {
    ($logger:ident, $log_data:ty) => {
        use executors::threadpool_executor::ThreadPoolExecutor;
        use executors::Executor;
        use lazy_static::lazy_static;
        use std::sync::Mutex;
        use knockoff_logging::knockoff_logging::logging_facade::LoggingFacade;
        use std::future::Future;

        lazy_static! {
            pub static ref logger: $logger = {
                let text_file_logger_unwrapped = $logger::new_from_file();
                text_file_logger_unwrapped.unwrap()
            };
            pub static ref executor: Mutex<ThreadPoolExecutor> = Mutex::new(ThreadPoolExecutor::new(2));
        }

        pub struct AsyncLoggingExecutor {
            executor: Mutex<ThreadPoolExecutor>
        }

        pub struct StandardLoggingFacade;

        impl LoggingFacade<$log_data, $logger> for StandardLoggingFacade {
            type LogFormatterType = StandardLogFormatter;
            type LoggerArgsType = TextFileLoggerArgs;

            fn get_logger() -> &'static $logger {
                &logger
            }
        }

    }
}

#[macro_export]
macro_rules! use_logging {
    () => {
        use knockoff_logging::knockoff_logging::log_level::{LogLevel, LogLevels};
        use knockoff_logging::knockoff_logging::text_file_logging::{TextFileLogger, TextFileLoggerArgs};
        use knockoff_logging::knockoff_logging::standard_formatter::{StandardLogData, StandardLogFormatter};
        use knockoff_logging::knockoff_logging::logger::Logger;
        use crate::executor;
        use crate::StandardLoggingFacade;
        use executors::Executor;
        use knockoff_logging::knockoff_logging::logging_facade::LoggingFacade;
    }
}

#[macro_export]
macro_rules! initialize_log {
    () => {

        #[macro_export]
        macro_rules! log {
            ($log_level:expr, $message:expr, $trace_id:expr) => {
                let _ = executor.lock()
                    .and_then(|exec| {
                        exec.execute(|| StandardLoggingFacade::get_logger().log($log_level, $message, $trace_id));
                        Ok(())
                    })
                    .or_else(|err| {
                        println!("Failed to unlock logger executor pool {}!", err.to_string());
                        Err(err)
                    });
            }
        }

        #[macro_export]
        macro_rules! log_info {
            ($message:expr, $trace_id:expr) => {
                executor.lock()
                    .map(|exec|  exec.execute(|| StandardLoggingFacade::get_logger().log(LogLevel::Info, $message, $trace_id)))
                    .or_else(|err| {
                        println!("Failed to unlock logger executor pool {}!", err.to_string());
                        Err(err)
                    });
            }
        }

        #[macro_export]
        macro_rules! log_message {
            ($message:expr) => {
                executor.lock()
                    .map(|exec|  exec.execute(|| StandardLoggingFacade::get_logger().log(LogLevel::Info, $message, "1".to_string())))
                    .or_else(|err| {
                        println!("Failed to unlock logger executor pool {}!", err.to_string());
                        Err(err)
                    });
            }
        }
    }
}


