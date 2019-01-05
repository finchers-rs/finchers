use {
    crate::{
        endpoint::{
            ActionContext, //
            ApplyContext,
            Endpoint,
            EndpointAction,
            IsEndpoint,
            Preflight,
        },
        error::Error,
    },
    futures::Poll,
};

#[allow(missing_docs)]
#[derive(Debug, Copy, Clone)]
pub struct MapErr<E, F> {
    pub(super) endpoint: E,
    pub(super) f: F,
}

impl<E: IsEndpoint, F> IsEndpoint for MapErr<E, F> {}

impl<E, F, Bd, U> Endpoint<Bd> for MapErr<E, F>
where
    E: Endpoint<Bd>,
    F: Fn(E::Error) -> U + Clone,
    U: Into<Error>,
{
    type Output = E::Output;
    type Error = U;
    type Action = MapErrAction<E::Action, F>;

    fn action(&self) -> Self::Action {
        MapErrAction {
            action: self.endpoint.action(),
            f: self.f.clone(),
        }
    }
}

#[derive(Debug)]
pub struct MapErrAction<A, F> {
    action: A,
    f: F,
}

impl<A, F, Bd, U> EndpointAction<Bd> for MapErrAction<A, F>
where
    A: EndpointAction<Bd>,
    F: Fn(A::Error) -> U,
    U: Into<Error>,
{
    type Output = A::Output;
    type Error = U;

    fn preflight(
        &mut self,
        cx: &mut ApplyContext<'_>,
    ) -> Result<Preflight<Self::Output>, Self::Error> {
        self.action.preflight(cx).map_err(|e| (self.f)(e))
    }

    fn poll_action(&mut self, cx: &mut ActionContext<'_, Bd>) -> Poll<Self::Output, Self::Error> {
        self.action.poll_action(cx).map_err(|e| (self.f)(e))
    }
}
