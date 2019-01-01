use std::marker::PhantomData;

use crate::common::{Func, Tuple};
use crate::endpoint::{ApplyContext, ApplyResult, Endpoint};
use crate::error::Error;
use crate::future::{Context, EndpointFuture, Poll};

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

impl<E, F> Wrapper<E> for Map<E::Output, F>
where
    E: Endpoint,
    F: Func<E::Output> + Clone,
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

impl<E, F> Endpoint for MapEndpoint<E, F>
where
    E: Endpoint,
    F: Func<E::Output> + Clone,
{
    type Output = (F::Out,);
    type Future = MapFuture<E::Future, F>;

    #[inline]
    fn apply(&self, ecx: &mut ApplyContext<'_>) -> ApplyResult<Self::Future> {
        Ok(MapFuture {
            future: self.endpoint.apply(ecx)?,
            f: Some(self.f.clone()),
        })
    }
}

#[derive(Debug)]
pub struct MapFuture<T, F> {
    future: T,
    f: Option<F>,
}

impl<T, F> EndpointFuture for MapFuture<T, F>
where
    T: EndpointFuture,
    F: Func<T::Output>,
    T::Output: Tuple,
{
    type Output = (F::Out,);

    fn poll_endpoint(&mut self, cx: &mut Context<'_>) -> Poll<Self::Output, Error> {
        let item = futures::try_ready!(self.future.poll_endpoint(cx));
        let f = self.f.take().expect("this future has already polled.");
        Ok((f.call(item),).into())
    }
}
