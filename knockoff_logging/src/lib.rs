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
    use std::{env, future, task, thread};
    use std::fs::File;
    use std::future::join;
    use std::io::Read;
    use std::ops::Add;
    use std::path::{Path, PathBuf};
    use executors::{Executor, futures_executor, JoinHandle};
    use executors::threadpool_executor::ThreadPoolExecutor;
    use crate::{initialize_log, initialize_logger};
    use lazy_static::lazy_static;
    use crate::knockoff_logging::log_level::LogLevel;
    use crate::knockoff_logging::logger::Logger;
    use crate::knockoff_logging::standard_formatter::{StandardLogData, StandardLogFormatter};
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::thread::spawn;
    use std::time::{Duration, Instant};
    use wait_for::wait_for::wait_async::WaitFor;
    use crate::knockoff_logging::logging_facade::{LoggingFacade};
    use crate::knockoff_logging::text_file_logging::{TextFileLogger, TextFileLoggerArgs};
    use crate::knockoff_logging::log_format::LogData;
    use crate::knockoff_logging::logger::LoggerArgs;
    use crate::knockoff_logging::log_format::LogFormatter;

    initialize_logger!(TextFileLogger, StandardLogData<'a>, 'a);
    initialize_log!();

    #[test]
    fn test_text_logging() {
        let logging_path = create_log_path();
        log!(LogLevel::Info, "test message", "1");
        assert_test_message(logging_path, "test message");
    }

    #[test]
    fn test_logging_facade() {
        env::set_var("LOGGING_DIR", "/Users/hayde/IdeaProjects/rust-spring-knockoff/knockoff_logging/resources/log.txt");
        let facade = StandardLoggingFacade::get_logger();
    }

    #[test]
    fn test_logging_macro() {
        env::set_var("LOGGING_DIR", "/Users/hayde/IdeaProjects/rust-spring-knockoff/knockoff_logging/resources/log.txt");
        log!(LogLevel::Info, "test message 1", "1");
        assert_test_message(create_log_path(), "test message 1");
        log_info!("test message 2", "1");
        assert_test_message(create_log_path(), "test message 2");
    }

    #[test]
    fn test_logging_macro_concurrent() {
        env::set_var("LOGGING_DIR", "/Users/hayde/IdeaProjects/rust-spring-knockoff/knockoff_logging/resources/log.txt");
        let mut join_handles = vec![];

        let builder = thread::Builder::new();
        for i in 0..10000 {
            let builder = thread::Builder::new();
            let mut task = builder.spawn(|| {
                log!(LogLevel::Info, "test message 1", "1");
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
        env::set_var("LOGGING_DIR", "/Users/hayde/IdeaProjects/rust-spring-knockoff/knockoff_logging/resources");
        let logging_file_result = env::var("LOGGING_DIR");
        let logging_file = logging_file_result.or::<String>(Ok("/Users/hayde/IdeaProjects/rust-spring-knockoff/knockoff_logging/resources".to_string())).unwrap();
        let mut logging_path = Path::new(&logging_file).join("log.txt");
        if !logging_path.exists() {
            File::create(&logging_path);
        }
        logging_path
    }
}
