use std::marker::PhantomData;

use futures::{Future, Poll};

use crate::common::{Func, Tuple};
use crate::endpoint::{ApplyContext, ApplyResult, Endpoint};
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
    fn apply(&'a self, ecx: &mut ApplyContext<'_>) -> ApplyResult<Self::Future> {
        Ok(MapFuture {
            future: self.endpoint.apply(ecx)?,
            f: Some(&self.f),
        })
    }
}

#[derive(Debug)]
pub struct MapFuture<'a, T, F> {
    future: T,
    f: Option<&'a F>,
}

impl<'a, T, F> Future for MapFuture<'a, T, F>
where
    T: Future<Error = Error>,
    T::Item: Tuple,
    F: Func<T::Item> + 'a,
{
    type Item = (F::Out,);
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let item = futures::try_ready!(self.future.poll());
        let f = self.f.take().expect("this future has already polled.");
        Ok((f.call(item),).into())
    }
}
