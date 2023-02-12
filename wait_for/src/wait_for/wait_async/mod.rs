use std::ops::Add;
use std::time::{Duration, Instant};

pub struct WaitFor {
}

impl WaitFor {
    pub fn wait_for<T>(duration: Duration, exec: &dyn Fn() -> T, matcher: &dyn Fn(T) -> bool) -> bool {
        let now = Instant::now();
        while now.add(duration) > Instant::now() {
            let next = exec();
            if matcher(next) {
                return true;
            }
        }
        false
    }
}
