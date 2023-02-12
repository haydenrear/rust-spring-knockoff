#[macro_export]
macro_rules! initialize_logger {
    () => {

        use crate::knockoff_logging::standard_formatter::StandardLogData;
        use crate::knockoff_logging::log_format::LogData;
        use crate::knockoff_logging::logger::LoggerArgs;
        use crate::knockoff_logging::log_format::LogFormatter;

        // TODO:
        lazy_static! {
            pub static ref text_file_logger: TextFileLogger = {
                let text_file_logger_unwrapped = TextFileLogger::new_from_file();
                text_file_logger_unwrapped.unwrap()
            };
            pub static ref executor: Arc<Mutex<ThreadPoolExecutor>> = Arc::new(Mutex::new(ThreadPoolExecutor::new(5)));
        }

        pub struct StandardLoggingFacade {
        }

        impl <'a> LoggingFacade<StandardLogData<'a>, TextFileLogger> for StandardLoggingFacade {
            type LogFormatterType = StandardLogFormatter;
            type LoggerArgsType = TextFileLoggerArgs;

            fn get_logger() -> &'static TextFileLogger {
                &text_file_logger
            }
        }

        pub struct LoggingProvider {
        }

        impl LoggingProvider {
            pub fn get_logging_facade<'a, FACADE, T, A, F, L>() -> &'a FACADE
            where
                T: LogData,
                A: LoggerArgs,
                F: LogFormatter<T>,
                L: Logger<T, LogFormatterType=F, LoggerArgsType=A>,
                FACADE: LoggingFacade<T, L, LogFormatterType=F, LoggerArgsType=A>
            {
                todo!()
            }
        }

    }
}

#[macro_export]
macro_rules! log {
    ($log_level:expr, $message:expr, $trace_id:expr) => {
        let _ = executor.lock()
            .and_then(|exec| {
                exec.execute(|| text_file_logger.log($log_level, $message, $trace_id));
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
            .map(|exec|  exec.execute(|| text_file_logger.log(LogLevel::Info, $message, $trace_id)))
            .or_else(|err| {
                println!("Failed to unlock logger executor pool {}!", err.to_string());
                Err(err)
            });
    }
}

