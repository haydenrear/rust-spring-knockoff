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
    pub mod default_logging;
}

#[cfg(test)]
mod test {
    use std::any::Any;
    use std::{env, future, task, thread};
    use std::fs::File;
    use std::future::join;
    use std::io::Read;
    use std::ops::Add;
    use std::path::{Path, PathBuf};
    use executors::{futures_executor, JoinHandle};
    use crate::{initialize_log, initialize_logger, log_message, use_logging, create_logger_expr};
    use crate::knockoff_logging::log_level::LogLevel;
    use crate::knockoff_logging::logger::Logger;
    use crate::knockoff_logging::standard_formatter::{StandardLogData, StandardLogFormatter};
    use std::sync::Arc;
    use std::thread::spawn;
    use std::time::{Duration, Instant};
    use wait_for::wait_for::wait_async::WaitFor;
    use crate::knockoff_logging::logging_facade::{LoggingFacade};
    use crate::knockoff_logging::text_file_logging::{TextFileLoggerImpl, TextFileLoggerArgs};
    use crate::knockoff_logging::log_format::LogData;
    use crate::knockoff_logging::logger::LoggerArgs;
    use crate::knockoff_logging::log_format::LogFormatter;
    use crate::knockoff_logging::text_file_logging::TextFileLogger;
    use crate::executor;
    use executors::Executor;

    initialize_logger!(TextFileLoggerImpl, StandardLogData, concat!(project_directory!(), "log_out/knockoff_logging_test.log"));
    initialize_log!();

    #[test]
    fn test_text_logging() {
        let logging_path = create_log_path();
        let facade = StandardLoggingFacade::get_logger();
        log!(LogLevel::Info, "test_mod message".to_string(), "1".to_string());
        assert_test_message(logging_path, "test_mod message");
    }

    #[test]
    fn test_logging_facade() {
        let facade = StandardLoggingFacade::get_logger();
    }

    #[test]
    fn test_log_message() {
        log_message!("test_mod message {}", "this_message");
        assert_test_message(create_log_path(), "test_mod message this_message");
    }

    #[test]
    fn test_logging_macro() {
        log!(LogLevel::Info, "test_mod message 1".to_string(), "1".to_string());
        assert_test_message(create_log_path(), "test_mod message 1");
        log_info!("test_mod message 2".to_string(), "1".to_string());
        assert_test_message(create_log_path(), "test_mod message 2");
    }

    #[test]
    fn test_logging_macro_concurrent() {
        let mut join_handles = vec![];

        let builder = thread::Builder::new();
        for i in 0..10000 {
            let builder = thread::Builder::new();
            let mut task = builder.spawn(|| {
                log!(LogLevel::Info, "test_mod message 1".to_string(), "1".to_string());
            }).unwrap();
            join_handles.push(task);
        }

        await_tasks(join_handles);
    }

    fn await_tasks(join_handles: Vec<thread::JoinHandle<()>>) {
        let mut all_complete = false;
        while !all_complete {
            for j in join_handles.iter() {
                if all_complete {
                    break;
                }
                if !j.is_finished() {
                    continue;
                }
                all_complete = true;
            }
            if all_complete {
                break;
            }

            thread::sleep(Duration::from_millis(30));
        }
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

    fn create_log_path() -> PathBuf {
        let logging_file_result = env::var("KNOCKOFF_LOGGING_TEST_FILE");
        let logging_file = logging_file_result.or::<String>(Ok(
            "/Users/hayde/IdeaProjects/rust-spring-knockoff/log_out/knockoff_logging_test.log".to_string())
        ).unwrap();
        let mut logging_path = Path::new(&logging_file);
        if !logging_path.exists() {
            File::create(&logging_path).unwrap();
        }
        logging_path.to_path_buf()
    }
}
