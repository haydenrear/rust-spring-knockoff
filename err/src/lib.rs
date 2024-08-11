use std::error::Error;
use std::io::ErrorKind;
use std::marker::PhantomData;
use knockoff_logging::error;

use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use knockoff_helper::project_directory;
import_logger_root!("lib.rs", concat!(project_directory!(), "/log_out/log_err.log"));


lazy_static! {
    pub static ref LOG_ERR: LogThisValue = LogThisValue {};
}

pub trait LogThis<T: Error> : Send + Sync {
}

#[derive(Copy, Clone)]
pub struct LogThisValue {
}

#[derive(Copy, Clone)]
pub struct LogWithPrepend<'a, T: Error> {
    to_prepend: &'a str,
    phantom_data: PhantomData<T>
}

pub trait CallOnce<T: Error> {
    fn call_once(self, args: (T,)) -> T;
}

impl<T: Error> CallOnce<T> for LogThisValue {
    fn call_once(self, args: (T,)) -> T {
        error!("{}",args.0);
        args.0
    }
}

impl<'a, T: Error> CallOnce<T> for LogWithPrepend<'a, T> {
    fn call_once(self, args: (T,)) -> T {
        error!("{}{}", self.to_prepend, args.0);
        args.0
    }
}

impl<T: Error + Send + Sync> LogThis<T> for LogThisValue {
}


impl<'a, T: Error + Sync + Send> LogThis<T> for LogWithPrepend<'a, T> {
}

#[test]
fn do_test() {
    let e: std::io::Error = std::io::Error::new(ErrorKind::AddrInUse, "");
    LOG_ERR(e);

    let out = log_err("hello")(std::io::Error::new(ErrorKind::AddrInUse, ""));
}

pub fn log_err<T: Error>(prepend: &str) -> LogWithPrepend<T> {
    LogWithPrepend {to_prepend: prepend, phantom_data: PhantomData::default()}
}
