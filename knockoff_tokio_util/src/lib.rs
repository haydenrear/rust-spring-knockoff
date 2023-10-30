use std::future::Future;
use std::hash::Hash;
use tokio::runtime::{Handle, Runtime};
use std::collections::{HashMap, HashSet};

pub fn run_blocking<F: Future>(fn_to_run: F) -> F::Output {
    get_create_runtime(fn_to_run)
}

pub fn get_create_runtime<F: Future>(fn_to_run: F) -> F::Output {
    let handle = Handle::try_current();
    return if handle.is_ok() {
        handle.unwrap().block_on(fn_to_run)
    } else {
        let out = Runtime::new().map(|runtime| {
            runtime.block_on(fn_to_run)
        });
        if out.is_err() {
            panic!("Could not run future!");
        }
        out.unwrap()
    }
}
