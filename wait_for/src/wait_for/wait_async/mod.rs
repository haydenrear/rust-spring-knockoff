use std::error::Error;
use std::future;
use std::future::{Future, PollFn};
use std::marker::PhantomData;
use std::ops::Add;
use std::pin::{Pin, pin};
use std::sync::Arc;
use std::sync::mpsc::RecvTimeoutError::Timeout;
use std::task::{Context, Poll};
use std::thread::sleep;
use std::time::{Duration, Instant};
use time::timeout;
use tokio::task::JoinHandle;
use tokio::{task, task_local, time};
use tokio::time::error::Elapsed;

#[derive(Default)]
pub struct WaitFor<U: Send + Sync> {
    pub t: Option<U>
}

#[derive(Default)]
pub struct Finished;

#[derive(Default)]
pub struct TimedOut;

pub trait TimedOutT: Send + Sync {}
impl TimedOutT for TimedOut{}
impl TimedOutT for Finished {}

/// Function returns an option if it is ready and None if it is not, but it returns back to the
/// caller. so that it can continue and be called multiple times.
#[derive(Default)]
pub struct Poller<F, T, FUT, DataProviderT>
    where
        F: Fn(Arc<DataProviderT>) -> FUT,
        FUT: Future<Output=Option<T>>,
        DataProviderT: Send + Sync,
        T: Send + Sync
{
    f: F,
    data_provider: Arc<DataProviderT>,
    p: PhantomData<T>,
    fut: PhantomData<FUT>
}

pub trait PollingFuture<T: Send + Sync, DataProviderT: Send + Sync>: Future<Output=Option<T>> {
    async fn do_poll(&self) -> Option<T>;
}

impl<F, T, FUT, DataProviderT> Future for Poller<F, T, FUT, DataProviderT>
    where
        F: Fn(Arc<DataProviderT>) -> FUT,
        FUT: Future<Output=Option<T>>,
        DataProviderT: Send + Sync,
        T: Send + Sync
{
    type Output = Option<T>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let polled = future::poll_fn(|ctx| {
            let polled = pin!(self.do_poll()).as_mut().poll(ctx);
            if let Poll::Ready(Some(t)) = polled {
                Poll::Ready(t)
            } else {
                Poll::Pending
            }
        });
        if let Poll::Ready(t) = pin!(polled).as_mut().poll(cx) {
            Poll::Ready(Some(t))
        } else {
            Poll::Pending
        }
    }
}

impl<F, T, FUT, DataProviderT> PollingFuture<T, DataProviderT> for Poller<F, T, FUT, DataProviderT>
where
    F: Fn(Arc<DataProviderT>) -> FUT,
    FUT: Future<Output=Option<T>>,
    DataProviderT: Send + Sync,
    T: Sync + Send
{
    async fn do_poll(&self) -> Option<T> {
        let polled: FUT = (self.f)(self.data_provider.clone());
        let polled_value = polled.await;
        if polled_value.as_ref().is_some() {
            Some(polled_value.unwrap())
        } else {
            None
        }
    }
}


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
