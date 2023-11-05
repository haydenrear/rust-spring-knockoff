use std::sync::{MutexGuard, PoisonError};
use crate::LogLevel;
use crate::standard_formatter::StandardLogData;
use crate::text_file_logging::TextFileLoggerImpl;

#[macro_export]
macro_rules! initialize_logger {
    () => {
        create_logger_expr!(TextFileLoggerImpl, StandardLogData, TextFileLoggerImpl::new_from_file_dir(concat!(project_directory!(), "/log_out/default_log.log")));
    };
    ($log_file:expr) => {
        create_logger_expr!(TextFileLoggerImpl, StandardLogData, TextFileLoggerImpl::new_from_file_dir($log_file));
    };
    ($logger:ident, $log_data:ty) => {
        create_logger_expr!($logger, $log_data, $logger::new_from_file());
    };
    ($logger:ident, $log_data:ty, $log_file:literal) => {
        create_logger_expr!($logger, $log_data, $logger::new_from_file_dir($log_file));
    };
    ($logger:ident, $log_data:ty, $log_file:expr) => {
        create_logger_expr!($logger, $log_data, $logger::new_from_file_dir($log_file));
    };
    ($logger:ident, $log_data:ty, $log_file:literal) => {
        create_logger_expr!($logger, $log_data, $logger::new_from_file_dir($log_file));
    };
    ($logger:ident, $log_data:ty, $log_file:literal) => {
        create_logger_expr!($logger, $log_data, $logger::new_from_file_dir($log_file));
    };
    ($logger:ident, $log_data:ty) => {
        create_logger_expr!($logger, $log_data, $logger::new_from_file());
    };
    ($logger:ident, $log_data:ty) => {
        create_logger_expr!($logger, $log_data, $logger::new_from_file());
    };
}

#[macro_export]
macro_rules! import_logger_root {
    ($package:literal) => {
        initialize_logger!();

        pub struct StandardLoggingFacade;

        impl LoggingFacade<StandardLogData, TextFileLoggerImpl> for StandardLoggingFacade {
            type LogFormatterType = StandardLogFormatter;
            type LoggerArgsType = TextFileLoggerArgs;

            fn get_logger() -> &'static Mutex<TextFileLoggerImpl> {
                &logger_lazy
            }
            fn package() -> &'static str {
                $package
            }
        }
    };
    ($package:literal, $log_file:expr) => {
        initialize_logger!($log_file);
        pub struct StandardLoggingFacade;

        impl LoggingFacade<StandardLogData, TextFileLoggerImpl> for StandardLoggingFacade {
            type LogFormatterType = StandardLogFormatter;
            type LoggerArgsType = TextFileLoggerArgs;

            fn get_logger() -> &'static Mutex<TextFileLoggerImpl> {
                &logger_lazy
            }
            fn package() -> &'static str {
                $package
            }
        }
    };
}


#[macro_export]
macro_rules! create_logger_expr {
    ($logger:ident, $log_data:ty, $create_logger:expr) => {

        lazy_static! {
            pub static ref logger_lazy: Mutex<$logger> = {
                let text_file_logger_unwrapped = $create_logger;
                Mutex::new(text_file_logger_unwrapped.unwrap() as $logger)
            };
        }
    }
}

#[macro_export]
macro_rules! import_logger {
    ($logger:ident, $log_data:ty, $package:literal) => {
        impl LoggingFacade<$log_data, $logger> for StandardLoggingFacade {
            type LogFormatterType = StandardLogFormatter;
            type LoggerArgsType = TextFileLoggerArgs;

            fn get_logger() -> &'static Mutex<$logger> {
                &logger_lazy
            }
            fn package(&self) -> &'static str {
                $package
            }
        }

    };
    ($package:literal) => {

        pub struct StandardLoggingFacade;

        impl LoggingFacade<StandardLogData, TextFileLoggerImpl> for StandardLoggingFacade {
            type LogFormatterType = StandardLogFormatter;
            type LoggerArgsType = TextFileLoggerArgs;

            fn get_logger() -> &'static Mutex<TextFileLoggerImpl> {
                &logger_lazy
            }
            fn package() -> &'static str {
                $package
            }
        }

    };
    ($package:literal, $root:literal) => {
        pub struct StandardLoggingFacade;

        impl LoggingFacade<StandardLogData, TextFileLoggerImpl> for StandardLoggingFacade {
            type LogFormatterType = StandardLogFormatter;
            type LoggerArgsType = TextFileLoggerArgs;

            fn get_logger() -> &'static Mutex<TextFileLoggerImpl> {
                &logger_lazy
            }
            fn package() -> &'static str {
                $package
            }
        }

    };
}


#[macro_export]
macro_rules! log {
    ($log_level:expr, $message:expr, $trace_id:expr) => {
        StandardLoggingFacade::get_logger()
            .lock().as_mut().map(|l| l.log($log_level, $message, $trace_id));
    }
}

#[macro_export]
macro_rules! log_info {
    ($message:expr, $trace_id:expr) => {
        StandardLoggingFacade::get_logger()
            .lock().as_mut().map(|l| l.log(LogLevel::Info, $message, $trace_id));
    }
}

#[macro_export]
macro_rules! log_message {
    ($($arg:tt)*) => {
        let message = format!($($arg)*);
        StandardLoggingFacade::get_logger()
            .lock().as_mut().map(|l| l.log(LogLevel::Info, message, StandardLoggingFacade::package().to_string()))
    };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        let message = format!($($arg)*);
        StandardLoggingFacade::get_logger()
            .lock().as_mut().map(|l| l.log(LogLevel::Info, message, StandardLoggingFacade::package().to_string()))
    };
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        let message = format!($($arg)*);
        StandardLoggingFacade::get_logger()
            .lock().as_mut().map(|l| l.log(LogLevel::Debug, message, StandardLoggingFacade::package().to_string()))
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        let message = format!($($arg)*);
        StandardLoggingFacade::get_logger()
            .lock()
            .as_mut()
            .map(|l| l.log(LogLevel::Error, message, StandardLoggingFacade::package().to_string()))
    };
}
