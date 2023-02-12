pub mod knockoff_logging {
    pub mod logger;
    pub mod text_file_logging;
    pub mod log_level;
    pub mod log_format;
    pub mod standard_formatter;
    pub mod print_logging;
    pub mod aggregate_logging;
}

#[cfg(test)]
mod test {
    use std::env;
    use std::fs::File;
    use std::io::Read;
    use std::path::{Path, PathBuf};
    use crate::knockoff_logging::log_level::LogLevel;
    use crate::knockoff_logging::logger::Logger;
    use crate::knockoff_logging::standard_formatter::StandardLogFormatter;
    use crate::knockoff_logging::text_file_logging::{TextFileLogger, TextFileLoggerArgs};

    #[test]
    fn test_text_logging() {
        let logging_path = create_log_path();
        log_test_message(&logging_path);
        assert_test_message(logging_path);
    }

    fn assert_test_message(logging_path: PathBuf) {
        let file = File::open(logging_path);
        assert!(file.is_ok());
        let mut out = "".to_string();
        file.unwrap().read_to_string(&mut out);
        assert!(out.as_str().contains("test message"));
    }

    fn log_test_message(logging_path: &PathBuf) {
        let logger_opt = TextFileLoggerArgs::new(&logging_path)
            .map(|file_args| TextFileLogger::new(file_args));

        assert!(logger_opt.is_some());

        let logger = logger_opt.unwrap();
        logger.log(LogLevel::Info, "test message", "1");
    }

    fn create_log_path() -> PathBuf {
        let logging_file_result = env::var("TEST_LOGGING_DIR");
        let logging_file = logging_file_result.or::<String>(Ok("/Users/hayde/IdeaProjects/rust-spring-knockoff/knockoff_logging/resources".to_string())).unwrap();
        let mut logging_path = Path::new(&logging_file).join("log.txt");
        logging_path
    }
}
