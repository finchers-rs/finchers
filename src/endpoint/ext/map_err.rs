use {
    crate::{
        action::{
            ActionContext, //
            EndpointAction,
            Preflight,
            PreflightContext,
        },
        endpoint::{Endpoint, IsEndpoint},
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

impl<E, F, Bd, R> Endpoint<Bd> for MapErr<E, F>
where
    E: Endpoint<Bd>,
    F: Fn(Error) -> R + Clone,
    R: Into<Error>,
{
    type Output = E::Output;
    type Action = MapErrAction<E::Action, F>;

    fn action(&self) -> Self::Action {
        MapErrAction {
            action: self.endpoint.action(),
            f: self.f.clone(),
        }
    }
}

#[allow(missing_debug_implementations)]
pub struct MapErrAction<Act, F> {
    action: Act,
    f: F,
}

impl<Act, F, Bd, R> EndpointAction<Bd> for MapErrAction<Act, F>
where
    Act: EndpointAction<Bd>,
    F: Fn(Error) -> R,
    R: Into<Error>,
{
    type Output = Act::Output;

    fn preflight(
        &mut self,
        cx: &mut PreflightContext<'_>,
    ) -> Result<Preflight<Self::Output>, Error> {
        self.action
            .preflight(cx)
            .map_err(|err| (self.f)(err).into())
    }

    fn poll_action(&mut self, cx: &mut ActionContext<'_, Bd>) -> Poll<Self::Output, Error> {
        self.action
            .poll_action(cx)
            .map_err(|err| (self.f)(err).into())
    }
}
