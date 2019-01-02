use crate::common::{Func, Tuple};
use crate::endpoint::{ApplyContext, ApplyResult, Endpoint};
use crate::error::Error;
use crate::future::{Context, EndpointFuture, Poll};

#[allow(missing_docs)]
#[derive(Debug, Copy, Clone)]
pub struct Map<E, F> {
    pub(super) endpoint: E,
    pub(super) f: F,
}

impl<E, F, Bd> Endpoint<Bd> for Map<E, F>
where
    E: Endpoint<Bd>,
    F: Func<E::Output> + Clone,
{
    type Output = (F::Out,);
    type Future = MapFuture<E::Future, F>;

    #[inline]
    fn apply(&self, ecx: &mut ApplyContext<'_, Bd>) -> ApplyResult<Self::Future> {
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

impl<T, F, Bd> EndpointFuture<Bd> for MapFuture<T, F>
where
    T: EndpointFuture<Bd>,
    F: Func<T::Output>,
    T::Output: Tuple,
{
    type Output = (F::Out,);

    fn poll_endpoint(&mut self, cx: &mut Context<'_, Bd>) -> Poll<Self::Output, Error> {
        let item = futures::try_ready!(self.future.poll_endpoint(cx));
        let f = self.f.take().expect("this future has already polled.");
        Ok((f.call(item),).into())
    }
}
