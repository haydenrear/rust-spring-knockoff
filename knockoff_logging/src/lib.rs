#![feature(async_fn_in_trait)]
#![feature(future_join)]

pub mod knockoff_logging {
    pub mod logger;
    pub mod text_file_logging;
    pub mod log_level;
    pub mod log_format;
    pub mod standard_formatter;
    pub mod print_logging;
    pub mod aggregate_logging;
    pub mod logging_macro;
    pub mod logging_facade;
}

#[cfg(test)]
mod test {
    use std::any::Any;
    use std::env;
    use std::fs::File;
    use std::future::join;
    use std::io::Read;
    use std::ops::Add;
    use std::path::{Path, PathBuf};
    use executors::Executor;
    use executors::threadpool_executor::ThreadPoolExecutor;
    use crate::{initialize_logger, log, log_info};
    use lazy_static::lazy_static;
    use crate::knockoff_logging::log_level::LogLevel;
    use crate::knockoff_logging::logger::Logger;
    use crate::knockoff_logging::standard_formatter::StandardLogFormatter;
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::time::{Duration, Instant};
    use wait_for::wait_for::wait_async::WaitFor;
    use crate::knockoff_logging::logging_facade::{LoggingFacade};
    use crate::knockoff_logging::text_file_logging::{TextFileLogger, TextFileLoggerArgs};

    #[test]
    fn test_text_logging() {
        let logging_path = create_log_path();
        log_test_message(&logging_path, "test message");
        assert_test_message(logging_path, "test message");
    }

    #[test]
    fn test_logging_facade() {
        initialize_logger!();
        let facade = StandardLoggingFacade::get_logger();
    }

    #[test]
    fn test_logging_macro() {
        env::set_var("LOGGING_DIR", "/Users/hayde/IdeaProjects/rust-spring-knockoff/knockoff_logging/resources/log.txt");
        initialize_logger!();
        log!(LogLevel::Info, "test message 1", "1");
        assert_test_message(create_log_path(), "test message 1");
        log_info!("test message 2", "1");
        assert_test_message(create_log_path(), "test message 2");
    }

    fn assert_test_message(logging_path: PathBuf, test_message: &str) {
        let asserted = WaitFor::wait_for(Duration::from_millis(20000), &|| {
            let file = File::open(logging_path.clone());
            assert!(file.is_ok());
            let mut out = "".to_string();
            file.unwrap().read_to_string(&mut out);
            String::from(out)
        }, &|out_str| {
            out_str.as_str().contains(test_message)
        });
        assert!(asserted);
    }

    fn log_test_message(logging_path: &PathBuf, test_message: &str) {
        let logger_opt = TextFileLoggerArgs::new(&logging_path)
            .map(|file_args| TextFileLogger::new(file_args));

        assert!(logger_opt.is_some());

        let logger = logger_opt.unwrap();
        logger.log(LogLevel::Info, test_message, "1");
    }

    fn create_log_path() -> PathBuf {
        env::set_var("LOGGING_DIR", "/Users/hayde/IdeaProjects/rust-spring-knockoff/knockoff_logging/resources");
        let logging_file_result = env::var("LOGGING_DIR");
        let logging_file = logging_file_result.or::<String>(Ok("/Users/hayde/IdeaProjects/rust-spring-knockoff/knockoff_logging/resources".to_string())).unwrap();
        let mut logging_path = Path::new(&logging_file).join("log.txt");
        logging_path
    }
}
