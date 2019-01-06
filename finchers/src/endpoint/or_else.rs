use {
    crate::{
        endpoint::{
            ActionContext, //
            Endpoint,
            EndpointAction,
            IsEndpoint,
            Preflight,
            PreflightContext,
        },
        error::Error,
    },
    futures::{Future, IntoFuture, Poll},
};

#[allow(missing_docs)]
#[derive(Debug, Copy, Clone)]
pub struct OrElse<E, F> {
    pub(super) endpoint: E,
    pub(super) f: F,
}

impl<E: IsEndpoint, F> IsEndpoint for OrElse<E, F> {}

impl<E, F, Bd, R> Endpoint<Bd> for OrElse<E, F>
where
    E: Endpoint<Bd, Output = (R::Item,)>,
    F: Fn(E::Error) -> R + Clone,
    R: IntoFuture,
    R::Error: Into<Error>,
{
    type Output = (R::Item,);
    type Error = R::Error;
    type Action = OrElseAction<E::Action, R::Future, F>;

    fn action(&self) -> Self::Action {
        OrElseAction {
            action: self.endpoint.action(),
            f: self.f.clone(),
            in_flight: None,
        }
    }
}

#[allow(missing_debug_implementations)]
pub struct OrElseAction<Act, Fut, F> {
    action: Act,
    f: F,
    in_flight: Option<Fut>,
}

impl<Act, F, Bd, R> EndpointAction<Bd> for OrElseAction<Act, R::Future, F>
where
    Act: EndpointAction<Bd, Output = (R::Item,)>,
    F: Fn(Act::Error) -> R,
    R: IntoFuture,
    R::Error: Into<Error>,
{
    type Output = (R::Item,);
    type Error = R::Error;

    fn preflight(
        &mut self,
        cx: &mut PreflightContext<'_>,
    ) -> Result<Preflight<Self::Output>, Self::Error> {
        debug_assert!(self.in_flight.is_none());
        self.action.preflight(cx).or_else(|err| {
            self.in_flight = Some((self.f)(err).into_future());
            Ok(Preflight::Incomplete)
        })
    }

    fn poll_action(&mut self, cx: &mut ActionContext<'_, Bd>) -> Poll<Self::Output, Self::Error> {
        loop {
            if let Some(ref mut in_flight) = self.in_flight {
                return in_flight.poll().map(|x| x.map(|out| (out,)));
            }
            match self.action.poll_action(cx) {
                Ok(x) => return Ok(x),
                Err(err) => self.in_flight = Some((self.f)(err).into_future()),
            }
        }
    }
}
