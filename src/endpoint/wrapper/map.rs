use std::marker::PhantomData;
use std::pin::PinMut;

use futures_core::future::{Future, TryFuture};
use futures_core::task;
use futures_core::task::Poll;
use pin_utils::{unsafe_pinned, unsafe_unpinned};

use crate::common::{Func, Tuple};
use crate::endpoint::{Context, Endpoint, EndpointResult};
use crate::error::Error;

use super::Wrapper;

/// Create a wrapper for creating an endpoint which maps the output
/// to another type using the specified function.
pub fn map<T, F>(f: F) -> Map<T, F>
where
    T: Tuple,
    F: Func<T>,
{
    Map {
        f,
        _marker: PhantomData,
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct Map<T, F> {
    f: F,
    _marker: PhantomData<fn(T)>,
}

impl<'a, E, F> Wrapper<'a, E> for Map<E::Output, F>
where
    E: Endpoint<'a>,
    F: Func<E::Output> + 'a,
{
    type Output = (F::Out,);
    type Endpoint = MapEndpoint<E, F>;

    fn wrap(self, endpoint: E) -> Self::Endpoint {
        MapEndpoint {
            endpoint,
            f: self.f,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct MapEndpoint<E, F> {
    endpoint: E,
    f: F,
}

impl<'a, E, F> Endpoint<'a> for MapEndpoint<E, F>
where
    E: Endpoint<'a>,
    F: Func<E::Output> + 'a,
{
    type Output = (F::Out,);
    type Future = MapFuture<'a, E::Future, F>;

    #[inline]
    fn apply(&'a self, ecx: &mut Context<'_>) -> EndpointResult<Self::Future> {
        Ok(MapFuture {
            future: self.endpoint.apply(ecx)?,
            f: Some(&self.f),
        })
    }
}

#[derive(Debug)]
pub struct MapFuture<'a, T, F: 'a> {
    future: T,
    f: Option<&'a F>,
}

impl<'a, T, F> MapFuture<'a, T, F> {
    unsafe_pinned!(future: T);
    unsafe_unpinned!(f: Option<&'a F>);
}

impl<'a, T, F> Future for MapFuture<'a, T, F>
where
    T: TryFuture<Error = Error>,
    T::Ok: Tuple,
    F: Func<T::Ok> + 'a,
{
    type Output = Result<(F::Out,), Error>;

    fn poll(mut self: PinMut<'_, Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        match self.future().try_poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(result) => {
                let f = self.f().take().expect("this future has already polled.");
                Poll::Ready(result.map(|item| (f.call(item),)))
            }
        }
    }
}
