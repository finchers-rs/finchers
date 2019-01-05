use {
    crate::{
        common::Func,
        endpoint::{
            ActionContext, //
            Apply,
            ApplyContext,
            Endpoint,
            EndpointAction,
            IsEndpoint,
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

    fn apply(&self, ecx: &mut ApplyContext<'_>) -> Apply<Bd, Self> {
        let future = self.endpoint.apply(ecx)?;
        Ok(AndThenAction {
            state: State::First(future, self.f.clone()),
        })
    }
}

#[allow(missing_debug_implementations)]
enum State<F1, F2, F> {
    First(F1, F),
    Second(F2),
}

#[allow(missing_debug_implementations)]
pub struct AndThenAction<Act, Fut, F> {
    state: State<Act, Fut, F>,
}

impl<Act, F, R, Bd> EndpointAction<Bd> for AndThenAction<Act, R::Future, F>
where
    Act: EndpointAction<Bd>,
    F: Func<Act::Output, Out = R>,
    R: IntoFuture<Error = Act::Error>,
{
    type Output = (R::Item,);
    type Error = R::Error;

    fn poll_action(&mut self, cx: &mut ActionContext<'_, Bd>) -> Poll<Self::Output, Self::Error> {
        loop {
            self.state = match self.state {
                State::First(ref mut action, ref f) => {
                    let args = futures::try_ready!(action.poll_action(cx));
                    State::Second(f.call(args).into_future())
                }
                State::Second(ref mut future) => return future.poll().map(|x| x.map(|out| (out,))),
            };
        }
    }
}
