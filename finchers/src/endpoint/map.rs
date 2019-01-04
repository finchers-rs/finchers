use {
    crate::{
        common::{Func, Tuple},
        endpoint::{
            ActionContext, //
            ApplyContext,
            ApplyResult,
            Endpoint,
            EndpointAction,
            IsEndpoint,
        },
        error::Error,
    },
    futures::Poll,
};

#[allow(missing_docs)]
#[derive(Debug, Copy, Clone)]
pub struct Map<E, F> {
    pub(super) endpoint: E,
    pub(super) f: F,
}

impl<E: IsEndpoint, F> IsEndpoint for Map<E, F> {}

impl<E, F, Bd> Endpoint<Bd> for Map<E, F>
where
    E: Endpoint<Bd>,
    F: Func<E::Output> + Clone,
{
    type Output = (F::Out,);
    type Action = MapAction<E::Action, F>;

    #[inline]
    fn apply(&self, ecx: &mut ApplyContext<'_, Bd>) -> ApplyResult<Self::Action> {
        Ok(MapAction {
            future: self.endpoint.apply(ecx)?,
            f: Some(self.f.clone()),
        })
    }
}

#[derive(Debug)]
pub struct MapAction<T, F> {
    future: T,
    f: Option<F>,
}

impl<T, F, Bd> EndpointAction<Bd> for MapAction<T, F>
where
    T: EndpointAction<Bd>,
    F: Func<T::Output>,
    T::Output: Tuple,
{
    type Output = (F::Out,);

    fn poll_action(&mut self, cx: &mut ActionContext<'_, Bd>) -> Poll<Self::Output, Error> {
        let item = futures::try_ready!(self.future.poll_action(cx));
        let f = self.f.take().expect("this future has already polled.");
        Ok((f.call(item),).into())
    }
}
