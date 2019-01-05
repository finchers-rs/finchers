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
    std::marker::PhantomData,
};

#[allow(missing_docs)]
#[derive(Debug, Copy, Clone)]
pub struct ErrInto<E, U> {
    pub(super) endpoint: E,
    pub(super) _marker: PhantomData<fn() -> U>,
}

impl<E: IsEndpoint, U> IsEndpoint for ErrInto<E, U> {}

impl<E, Bd, U> Endpoint<Bd> for ErrInto<E, U>
where
    E: Endpoint<Bd>,
    E::Error: Into<U>,
    U: Into<Error>,
{
    type Output = E::Output;
    type Error = U;
    type Action = ErrIntoAction<E::Action, U>;

    fn action(&self) -> Self::Action {
        ErrIntoAction {
            action: self.endpoint.action(),
            _marker: PhantomData,
        }
    }
}

#[derive(Debug)]
pub struct ErrIntoAction<A, U> {
    action: A,
    _marker: PhantomData<fn() -> U>,
}

impl<A, Bd, U> EndpointAction<Bd> for ErrIntoAction<A, U>
where
    A: EndpointAction<Bd>,
    A::Error: Into<U>,
    U: Into<Error>,
{
    type Output = A::Output;
    type Error = U;

    #[inline]
    fn preflight(
        &mut self,
        cx: &mut ApplyContext<'_>,
    ) -> Result<Preflight<Self::Output>, Self::Error> {
        self.action.preflight(cx).map_err(Into::into)
    }

    #[inline]
    fn poll_action(&mut self, cx: &mut ActionContext<'_, Bd>) -> Poll<Self::Output, Self::Error> {
        self.action.poll_action(cx).map_err(Into::into)
    }
}
