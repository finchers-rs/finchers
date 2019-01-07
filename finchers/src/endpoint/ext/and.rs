use {
    crate::{
        common::Combine,
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
    type Action = AndAction<Bd, E1::Action, E2::Action>;

    fn action(&self) -> Self::Action {
        AndAction {
            f1: MaybeDone::Init(Some(self.e1.action())),
            f2: MaybeDone::Init(Some(self.e2.action())),
        }
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

impl<Bd, F1, F2> AndAction<Bd, F1, F2>
where
    F1: EndpointAction<Bd>,
    F2: EndpointAction<Bd>,
    F1::Output: Combine<F2::Output>,
{
    fn take_item(&mut self) -> Option<<F1::Output as Combine<F2::Output>>::Out> {
        let v1 = self.f1.take_item()?;
        let v2 = self.f2.take_item()?;
        Some(Combine::combine(v1, v2))
    }
}

impl<F1, F2, Bd> EndpointAction<Bd> for AndAction<Bd, F1, F2>
where
    F1: EndpointAction<Bd>,
    F2: EndpointAction<Bd>,
    F1::Output: Combine<F2::Output>,
{
    type Output = <F1::Output as Combine<F2::Output>>::Out;

    fn preflight(
        &mut self,
        cx: &mut PreflightContext<'_>,
    ) -> Result<Preflight<Self::Output>, Error> {
        let x1 = self.f1.preflight(cx)?;
        let x2 = self.f2.preflight(cx)?;
        if x1.is_completed() && x2.is_completed() {
            let out = self.take_item().expect("the value shoud be ready.");
            Ok(Preflight::Completed(out))
        } else {
            Ok(Preflight::Incomplete)
        }
    }

    fn poll_action(&mut self, cx: &mut ActionContext<'_, Bd>) -> Poll<Self::Output, Error> {
        futures::try_ready!(self.f1.poll_action(cx));
        futures::try_ready!(self.f2.poll_action(cx));
        let out = self.take_item().expect("the value should be ready.");
        Ok(out.into())
    }
}

#[derive(Debug)]
#[must_use = "futures do nothing unless polled."]
pub enum MaybeDone<Bd, F: EndpointAction<Bd>> {
    Init(Option<F>),
    InFlight(F),
    Ready(F::Output),
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

    fn preflight(
        &mut self,
        cx: &mut PreflightContext<'_>,
    ) -> Result<Preflight<Self::Output>, Error> {
        *self = match self {
            MaybeDone::Init(ref mut action) => {
                let mut action = action.take().unwrap();
                if let Preflight::Completed(output) = action.preflight(cx)? {
                    MaybeDone::Ready(output)
                } else {
                    MaybeDone::InFlight(action)
                }
            }
            _ => panic!("unexpected condition"),
        };

        Ok(Preflight::Incomplete)
    }

    fn poll_action(&mut self, cx: &mut ActionContext<'_, Bd>) -> Poll<Self::Output, Error> {
        loop {
            *self = match self {
                MaybeDone::Init(..) => panic!("The action has not yet initialized."),
                MaybeDone::Ready(..) => return Ok(Async::Ready(())),
                MaybeDone::InFlight(ref mut future) => {
                    let output = futures::try_ready!(future.poll_action(cx));
                    MaybeDone::Ready(output)
                }
                MaybeDone::Gone => panic!("The action has already been polled."),
            };
        }
    }
}
