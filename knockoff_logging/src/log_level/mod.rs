use std::collections::HashMap;
use std::iter::{IntoIterator, Iterator, Map};
use std::string::ToString;
use lazy_static::lazy_static;

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Hash)]
pub enum LogLevel {
    Debug = 0, Info = 1, Trace = 2, Warn = 3, Error = 4
}

lazy_static!(
    pub static ref LogLevels: HashMap<LogLevel, &'static str> = vec![
        (LogLevel::Debug, "Debug"),
        (LogLevel::Info, "Info"),
        (LogLevel::Trace, "Trace"),
        (LogLevel::Warn, "Warn"),
        (LogLevel::Error, "Error")
    ].into_iter().collect();
);
