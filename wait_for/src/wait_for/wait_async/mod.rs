use std::error::Error;
use std::future::Future;
use std::ops::{Add, Deref, DerefMut};
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, Instant};
use time::timeout;
use tokio::time;

mod timeout_future;

#[derive(Default)]
pub struct WaitFor<U: Send + Sync> {
    pub t: Option<U>
}

#[derive(Default)]
pub struct Finished;

#[derive(Default)]
pub struct TimedOut;

impl TimedOutT for TimedOut{}
impl TimedOutT for Finished {}


impl<U: Send + Sync> WaitFor<U> {
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

    pub async fn wait_for_matcher_async<T: Send + Sync>(duration: Duration,
                                                        matcher: &dyn Fn(T) -> bool,
                                                        future: impl Future<Output=T> + Send + 'static) -> bool {
        let timeout = timeout(duration, future);
        let awaited_timeout = timeout.await;
        match awaited_timeout {
            Ok(t) => {
                matcher(t)
            }
            Err(_) => {
                false
            }
        }
    }

    pub async fn wait_for_matcher_async_poll<T: Send + Sync>(
        duration: Duration,
        matcher: &dyn Fn(T) -> bool,
        future: impl Future<Output=T> + Send + 'static
    ) -> bool {
        let timeout = timeout(duration, future);
        let awaited_timeout = timeout.await;
        match awaited_timeout {
            Ok(t) => {
                matcher(t)
            }
            Err(_) => {
                false
            }
        }
    }

    pub async fn wait_for_matcher_async_multiple<'b, T: Send + Sync, V: Send + Sync>(
        wait: Arc<WaitFor<V>>,
        duration: Duration,
        matcher: &dyn Fn(T) -> bool,
        future_created: &dyn Fn(Arc<WaitFor<V>>) ->  Pin<Box<dyn Future<Output=T> + Send>>,
    ) -> bool {
        let now = Instant::now();
        while now.add(duration) > Instant::now() {
            let mut future_found = future_created(wait.clone());
            let created = future_found.await;
            if matcher(created) {
                return true;
            }
        }
        false
    }

    pub async fn wait_for_complete_async(duration: Duration,
                                                              future: impl Future<Output=()> + Send + 'static) -> bool {
        Self::wait_for_matcher_async(duration, &|()|  true, future).await
    }

}

#[tokio::test]
async fn test_async() {
    assert!(!WaitFor::<()>::wait_for_matcher_async(Duration::from_millis(1), &|()|  true, async {
        let mut counter = 0;
            while counter < 10000000000 as u64 {
                // needed so that it won't continue on forever.
                tokio::time::sleep(Duration::from_micros(1)).await;
                counter += 1;
            }
    }).await);
}

#[tokio::test]
async fn test_async_multiple() {
    let wait_value = Arc::new(WaitFor { t: String::from("hello").into() });
    let string_matcher = &|v: Option<String>| v.as_ref().is_some() && v.unwrap() == String::from("hello");
    assert!(WaitFor::<String>::wait_for_matcher_async_multiple(
        wait_value,
        Duration::from_secs(10),
        string_matcher,
        &|value| {
            let pinned_fn = Box::pin(async move {
                Some(String::from("hello"))
            });
            pinned_fn
        }).await);
}

pub trait TimedOutT: Send + Sync {}
