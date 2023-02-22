#[macro_export]
macro_rules! initialize_logger {
    ($logger:ident, $log_data:ty) => {
        create_logger_expr!($logger, $log_data, $logger::new_from_file(), 2);
    };
    ($logger:ident, $log_data:ty, $log_file:literal) => {
        create_logger_expr!($logger, $log_data, $logger::new_from_file_dir($log_file), 2);
    };
    ($logger:ident, $log_data:ty, $log_file:literal, $num_cores:literal) => {
        create_logger_expr!($logger, $log_data, $logger::new_from_file_dir($log_file), $num_cores);
    };
    ($logger:ident, $log_data:ty, $num_cores:literal) => {
        create_logger_expr!($logger, $log_data, $logger::new_from_file(), $num_cores);
    };
}

#[macro_export]
macro_rules! create_logger_expr {
    ($logger:ident, $log_data:ty, $create_logger:expr, $num_cores:literal) => {
        use executors::threadpool_executor::ThreadPoolExecutor;
        use std::sync::Mutex;
        use std::future::Future;
        use lazy_static::lazy_static;

        lazy_static! {
            pub static ref logger: $logger = {
                let text_file_logger_unwrapped = $create_logger;
                text_file_logger_unwrapped.unwrap()
            };
            pub static ref executor: Mutex<ThreadPoolExecutor> = Mutex::new(ThreadPoolExecutor::new($num_cores));
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

    };
}

#[macro_export]
macro_rules! use_logging {
    () => {
        use knockoff_logging::knockoff_logging::log_level::{LogLevel, LogLevels};
        use knockoff_logging::knockoff_logging::text_file_logging::{TextFileLogger, TextFileLoggerArgs, TextFileLoggerImpl};
        use knockoff_logging::knockoff_logging::standard_formatter::{StandardLogData, StandardLogFormatter};
        use knockoff_logging::knockoff_logging::logger::Logger;
        use knockoff_logging::knockoff_logging::logging_facade::LoggingFacade;
        use $crate::log_message;
        use executors::Executor;
    }
}

#[macro_export]
macro_rules! initialize_log {
    () => {

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

    };
}

#[macro_export]
macro_rules! log_message {
    ($($arg:tt)*) => {
        let message = format!($($arg)*);
        executor.lock()
            .map(|exec|  exec.execute(|| StandardLoggingFacade::get_logger().log(LogLevel::Info, message, "1".to_string())))
            .or_else(|err| {
                println!("Failed to unlock logger executor pool {}!", err.to_string());
                Err(err)
            });
    };
}
