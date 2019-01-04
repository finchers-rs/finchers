use {
    crate::{
        endpoint::{
            ActionContext, //
            Apply,
            ApplyContext,
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

    fn apply(&self, ecx: &mut ApplyContext<'_, Bd>) -> Apply<Bd, Self> {
        match self.endpoint.apply(ecx) {
            Ok(action) => Ok(OrElseAction {
                state: State::First(action, self.f.clone()),
            }),
            Err(err) => Ok(OrElseAction {
                state: State::Second((self.f)(err).into_future()),
            }),
        }
    }
}

#[allow(missing_debug_implementations)]
enum State<F1, F2, F> {
    First(F1, F),
    Second(F2),
}

#[allow(missing_debug_implementations)]
pub struct OrElseAction<Act, Fut, F> {
    state: State<Act, Fut, F>,
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

    fn poll_action(&mut self, cx: &mut ActionContext<'_, Bd>) -> Poll<Self::Output, Self::Error> {
        loop {
            self.state = match self.state {
                State::First(ref mut action, ref f) => match action.poll_action(cx) {
                    Ok(x) => return Ok(x),
                    Err(err) => State::Second(f(err).into_future()),
                },
                State::Second(ref mut future) => return future.poll().map(|x| x.map(|out| (out,))),
            };
        }
    }
}
