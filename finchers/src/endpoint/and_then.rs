use {
    crate::{
        common::Func,
        endpoint::{
            ActionContext, //
            Endpoint,
            EndpointAction,
            IsEndpoint,
            Preflight,
            PreflightContext,
        },
    },
    futures::{Future, IntoFuture, Poll},
};

#[allow(missing_docs)]
#[derive(Debug, Copy, Clone)]
pub struct AndThen<E, F> {
    pub(super) endpoint: E,
    pub(super) f: F,
}

impl<E: IsEndpoint, F> IsEndpoint for AndThen<E, F> {}

impl<E, F, Bd, R> Endpoint<Bd> for AndThen<E, F>
where
    E: Endpoint<Bd>,
    F: Func<E::Output, Out = R> + Clone,
    R: IntoFuture<Error = E::Error>,
{
    type Output = (R::Item,);
    type Error = R::Error;
    type Action = AndThenAction<E::Action, R::Future, F>;

    fn action(&self) -> Self::Action {
        AndThenAction {
            action: self.endpoint.action(),
            f: self.f.clone(),
            in_flight: None,
        }
    }
}

#[allow(missing_debug_implementations)]
pub struct AndThenAction<Act, Fut, F> {
    action: Act,
    f: F,
    in_flight: Option<Fut>,
}

impl<Act, F, R, Bd> EndpointAction<Bd> for AndThenAction<Act, R::Future, F>
where
    Act: EndpointAction<Bd>,
    F: Func<Act::Output, Out = R>,
    R: IntoFuture<Error = Act::Error>,
{
    type Output = (R::Item,);
    type Error = R::Error;

    fn preflight(
        &mut self,
        cx: &mut PreflightContext<'_>,
    ) -> Result<Preflight<Self::Output>, Self::Error> {
        debug_assert!(self.in_flight.is_none());
        if let Preflight::Completed(output) = self.action.preflight(cx)? {
            self.in_flight = Some(self.f.call(output).into_future());
        }
        Ok(Preflight::Incomplete)
    }

    fn poll_action(&mut self, cx: &mut ActionContext<'_, Bd>) -> Poll<Self::Output, Self::Error> {
        loop {
            if let Some(ref mut in_flight) = self.in_flight {
                return in_flight.poll().map(|x| x.map(|out| (out,)));
            }

            let args = futures::try_ready!(self.action.poll_action(cx));
            self.in_flight = Some(self.f.call(args).into_future());
        }
    }
}
