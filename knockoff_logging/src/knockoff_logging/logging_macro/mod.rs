#[macro_export]
macro_rules! initialize_logger {
    ($logger:ident, $log_data:ty, $lifetime:lifetime) => {

        lazy_static! {
            pub static ref logger: $logger = {
                let text_file_logger_unwrapped = $logger::new_from_file();
                text_file_logger_unwrapped.unwrap()
            };
            pub static ref executor: Mutex<ThreadPoolExecutor> = Mutex::new(ThreadPoolExecutor::new(5));
        }

        pub struct StandardLoggingFacade;

        impl <$lifetime> LoggingFacade<$log_data, $logger> for StandardLoggingFacade {
            type LogFormatterType = StandardLogFormatter;
            type LoggerArgsType = TextFileLoggerArgs;

            fn get_logger() -> &'static $logger {
                &logger
            }
        }

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
    }
}


