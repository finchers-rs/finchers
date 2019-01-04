use {
    crate::{
        common::Func,
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
    R: IntoFuture,
    R::Error: Into<Error>,
{
    type Output = (R::Item,);
    type Action = AndThenAction<E::Action, R::Future, F>;

    fn apply(&self, ecx: &mut ApplyContext<'_, Bd>) -> ApplyResult<Self::Action> {
        let future = self.endpoint.apply(ecx)?;
        Ok(AndThenAction {
            state: State::First(future, Some(self.f.clone())),
        })
    }
}

#[allow(missing_debug_implementations)]
enum State<F1, F2, F> {
    First(F1, Option<F>),
    Second(F2),
}

#[allow(missing_debug_implementations)]
pub struct AndThenAction<Act, Fut, F> {
    state: State<Act, Fut, F>,
}

impl<Act, Fut, F, R, Bd> EndpointAction<Bd> for AndThenAction<Act, Fut, F>
where
    Act: EndpointAction<Bd>,
    Fut: Future<Item = R::Item, Error = R::Error>,
    F: Func<Act::Output, Out = R>,
    R: IntoFuture<Future = Fut>,
    R::Error: Into<Error>,
{
    type Output = (R::Item,);

    fn poll_action(&mut self, cx: &mut ActionContext<'_, Bd>) -> Poll<Self::Output, Error> {
        loop {
            self.state = match self.state {
                State::First(ref mut action, ref mut f) => {
                    let args = futures::try_ready!(action.poll_action(cx));
                    let f = f.take().expect("unexpected condition");
                    State::Second(f.call(args).into_future())
                }
                State::Second(ref mut future) => {
                    return future
                        .poll()
                        .map(|x| x.map(|out| (out,)))
                        .map_err(Into::into)
                }
            };
        }
    }
}
