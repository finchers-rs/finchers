use finchers_core::HttpError;
use finchers_core::endpoint::{Context, Endpoint, task::{self, Future, Poll}};

pub fn new<E, F, T, R>(endpoint: E, f: F) -> TryAbort<E, F>
where
    E: Endpoint,
    F: FnOnce(E::Item) -> Result<T, R> + Clone + Send,
    R: HttpError,
{
    TryAbort { endpoint, f }
}

#[derive(Copy, Clone, Debug)]
pub struct TryAbort<E, F> {
    endpoint: E,
    f: F,
}

impl<E, F, T, R> Endpoint for TryAbort<E, F>
where
    E: Endpoint,
    F: FnOnce(E::Item) -> Result<T, R> + Clone + Send,
    R: HttpError,
{
    type Item = T;
    type Future = TryAbortFuture<E::Future, F>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Future> {
        let future = self.endpoint.apply(cx)?;
        Some(TryAbortFuture {
            future,
            f: Some(self.f.clone()),
        })
    }
}

#[derive(Debug)]
pub struct TryAbortFuture<T, F> {
    future: T,
    f: Option<F>,
}

impl<T, F, U, E> Future for TryAbortFuture<T, F>
where
    T: Future + Send,
    F: FnOnce(T::Item) -> Result<U, E> + Clone + Send,
    E: HttpError,
{
    type Item = U;

    fn poll(&mut self, cx: &mut task::Context) -> Poll<Self::Item> {
        let item = try_ready!(self.future.poll(cx));
        let f = self.f.take().expect("cannot resolve/reject twice");
        f(item).map_err(Into::into).map(Into::into)
    }
}
