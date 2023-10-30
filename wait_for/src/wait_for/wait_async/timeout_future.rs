use std::future::Future;
use std::pin::{Pin, pin};
use std::task::{Context, Poll};
use std::future;
use std::sync::Arc;
use std::marker::PhantomData;
use std::time::Instant;

/// Function returns an option if it is ready and None if it is not, but it returns back to the
/// caller. so that it can continue and be called multiple times.
#[derive(Default)]
pub struct TimeoutPoller<FutureProviderT, T, TimeoutFutureT, DataProviderT>
    where
        FutureProviderT: TimeoutFutureProvider<DataProviderT, TimeoutFutureT, T>,
        TimeoutFutureT: TimeoutFuture<T>,
        DataProviderT: DataProvider<T>,
        T: Send + Sync
{
    f: FutureProviderT,
    data_provider: Arc<DataProviderT>,
    p: PhantomData<T>,
    fut: PhantomData<TimeoutFutureT>
}

/// The polling future is the
pub trait PollingFuture<T: Send + Sync, DataProviderT: DataProvider<T>>: Future<Output=Option<T>> {
    async fn do_poll(&self) -> Option<T>;
}

/// Async provider of the next step to assert against.
pub trait DataProvider<T: Send + Sync>: Send + Sync {
    fn poll_data(&self, c: Option<&mut Context<'_>>) -> Option<T>;
    /// The Instant::now + Duration call to compare with, to see if time has run out
    fn end(&self) -> Arc<Instant>;
}

impl<TimeoutFutureProviderT, T, FutureT, DataProviderT> Future
for TimeoutPoller<TimeoutFutureProviderT, T, FutureT, DataProviderT>
    where
        TimeoutFutureProviderT: TimeoutFutureProvider<DataProviderT, FutureT, T>,
        FutureT: TimeoutFuture<T>,
        DataProviderT: DataProvider<T>,
        T: Send + Sync
{
    type Output = Option<T>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let polled = future::poll_fn(|ctx| {
            pin!(self.do_poll()).as_mut().poll(ctx)
        });
        if let Poll::Ready(t) = pin!(polled).as_mut().poll(cx) {
            Poll::Ready(t)
        } else {
            Poll::Pending
        }
    }
}


pub trait TimeoutFuture<T>: Future<Output=Option<T>>
where
    T: Send + Sync {}

pub trait TimeoutFutureProvider<DataProviderT, FUT, T>
where
    DataProviderT: DataProvider<T>,
    T: Send + Sync,
    FUT: TimeoutFuture<T>
{
    /// Create the next future for the next call. Must be broken into multiple futures so that
    /// long running operations don't hold up the thread (if a long running operation never gives
    /// back control to the caller then it will go on past the duration). Any time the inner await
    /// method exceeds the duration, it will continue until it ends.
    fn next_future(data: Arc<DataProviderT>) -> FUT;
}

pub struct TimeoutFutureImpl<DataProviderT: DataProvider<T>, T: Send + Sync> {
    data_provider: Arc<DataProviderT>,
    provided_data: PhantomData<T>,
    end: Arc<Instant>
}

impl<T: Send + Sync, DataProviderT: DataProvider<T>> Future
for TimeoutFutureImpl<DataProviderT, T> {
    type Output = Option<T>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(t) = self.data_provider.poll_data(cx.into()) {
            Poll::Ready(Some(t))
        }  else if *self.end > Instant::now() {
            Poll::Ready(None)
        } else {
            Poll::Pending
        }
    }
}

impl <T: Send + Sync, DataProviderT: DataProvider<T>> TimeoutFuture<T>
for TimeoutFutureImpl<DataProviderT, T> {}

impl<DataProviderT, T: Send + Sync> TimeoutFutureProvider<DataProviderT, TimeoutFutureImpl<DataProviderT, T>, T>
for TimeoutFutureImpl<DataProviderT, T>
where
    DataProviderT: DataProvider<T>,
{
    fn next_future(data: Arc<DataProviderT>) -> TimeoutFutureImpl<DataProviderT, T> {
        TimeoutFutureImpl {
            data_provider: data.clone(),
            provided_data: Default::default() ,
            end: data.end(),
        }
    }
}

/// Polling future is used to call the TimeoutFuture over and over again, allowing to await
///  the future without blocking, demanding that the control be given back to the caller by
///  creating another future. The future created calls the DataProvider again to get the next
///  value to assert with. This can be an async call to poll against with the same context
///  as this one in the poll.
impl<F, T, FUT, DataProviderT> PollingFuture<T, DataProviderT> for TimeoutPoller<F, T, FUT, DataProviderT>
where
    F: TimeoutFutureProvider<DataProviderT, FUT, T>,
    FUT: TimeoutFuture<T>,
    DataProviderT: DataProvider<T>,
    T: Sync + Send
{
    async fn do_poll(&self) -> Option<T> {
        let polled: FUT = F::next_future(self.data_provider.clone());
        let polled_value = polled.await;
        if polled_value.as_ref().is_some() {
            Some(polled_value.unwrap())
        } else {
            None
        }
    }
}
