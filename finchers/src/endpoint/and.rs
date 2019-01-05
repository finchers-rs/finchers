use {
    crate::{
        common::{Combine, Tuple},
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
    futures::{Async, Poll},
};

#[allow(missing_docs)]
#[derive(Copy, Clone, Debug)]
pub struct And<E1, E2> {
    pub(super) e1: E1,
    pub(super) e2: E2,
}

impl<E1: IsEndpoint, E2: IsEndpoint> IsEndpoint for And<E1, E2> {}

impl<E1, E2, Bd> Endpoint<Bd> for And<E1, E2>
where
    E1: Endpoint<Bd>,
    E2: Endpoint<Bd>,
    E1::Output: Combine<E2::Output>,
{
    type Output = <E1::Output as Combine<E2::Output>>::Out;
    type Error = Error;
    type Action = AndAction<Bd, E1::Action, E2::Action>;

    fn apply(&self, ecx: &mut ApplyContext<'_>) -> Apply<Bd, Self> {
        Ok(AndAction {
            f1: self
                .e1
                .apply(ecx)
                .map(MaybeDone::Pending)
                .map_err(Into::into)?,
            f2: self
                .e2
                .apply(ecx)
                .map(MaybeDone::Pending)
                .map_err(Into::into)?,
        })
    }
}

#[allow(missing_debug_implementations)]
pub struct AndAction<Bd, F1, F2>
where
    F1: EndpointAction<Bd>,
    F2: EndpointAction<Bd>,
{
    f1: MaybeDone<Bd, F1>,
    f2: MaybeDone<Bd, F2>,
}

impl<F1, F2, Bd> EndpointAction<Bd> for AndAction<Bd, F1, F2>
where
    F1: EndpointAction<Bd>,
    F2: EndpointAction<Bd>,
    F1::Output: Combine<F2::Output>,
    F2::Output: Tuple,
{
    type Output = <F1::Output as Combine<F2::Output>>::Out;
    type Error = Error;

    fn poll_action(&mut self, cx: &mut ActionContext<'_, Bd>) -> Poll<Self::Output, Error> {
        futures::try_ready!(self.f1.poll_action(cx).map_err(Into::into));
        futures::try_ready!(self.f2.poll_action(cx).map_err(Into::into));
        let v1 = self
            .f1
            .take_item()
            .expect("the future has already been polled.");
        let v2 = self
            .f2
            .take_item()
            .expect("the future has already been polled.");
        Ok(Combine::combine(v1, v2).into())
    }
}

#[derive(Debug)]
#[must_use = "futures do nothing unless polled."]
pub enum MaybeDone<Bd, F: EndpointAction<Bd>> {
    Ready(F::Output),
    Pending(F),
    Gone,
}

impl<Bd, F: EndpointAction<Bd>> MaybeDone<Bd, F> {
    pub fn take_item(&mut self) -> Option<F::Output> {
        match std::mem::replace(self, MaybeDone::Gone) {
            MaybeDone::Ready(output) => Some(output),
            _ => None,
        }
    }
}

impl<Bd, F: EndpointAction<Bd>> EndpointAction<Bd> for MaybeDone<Bd, F> {
    type Output = ();
    type Error = F::Error;

    fn poll_action(&mut self, cx: &mut ActionContext<'_, Bd>) -> Poll<Self::Output, Self::Error> {
        let polled = match self {
            MaybeDone::Ready(..) => return Ok(Async::Ready(())),
            MaybeDone::Pending(ref mut future) => future.poll_action(cx)?,
            MaybeDone::Gone => panic!("This future has already polled"),
        };
        match polled {
            Async::Ready(output) => {
                *self = MaybeDone::Ready(output);
                Ok(Async::Ready(()))
            }
            Async::NotReady => Ok(Async::NotReady),
        }
    }
}
