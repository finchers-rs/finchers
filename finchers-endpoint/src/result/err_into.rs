use finchers_core::endpoint::{Context, Endpoint};
use finchers_core::outcome::{self, Outcome, PollOutcome};
use std::marker::PhantomData;

#[derive(Debug, Copy, Clone)]
pub struct ErrInto<E, T> {
    endpoint: E,
    _marker: PhantomData<fn() -> T>,
}

pub fn new<E, U, A, B>(endpoint: E) -> ErrInto<E, U>
where
    E: Endpoint<Output = Result<A, B>>,
    B: Into<U>,
{
    ErrInto {
        endpoint,
        _marker: PhantomData,
    }
}

impl<E, A, B, U> Endpoint for ErrInto<E, U>
where
    E: Endpoint<Output = Result<A, B>>,
    B: Into<U>,
{
    type Output = Result<A, U>;
    type Outcome = ErrIntoOutcome<E::Outcome, U>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Outcome> {
        Some(ErrIntoOutcome {
            outcome: self.endpoint.apply(cx)?,
            _marker: PhantomData,
        })
    }
}

#[derive(Debug)]
pub struct ErrIntoOutcome<T, U> {
    outcome: T,
    _marker: PhantomData<fn() -> U>,
}

impl<T, U, A, B> Outcome for ErrIntoOutcome<T, U>
where
    T: Outcome<Output = Result<A, B>> + Send,
    B: Into<U>,
{
    type Output = Result<A, U>;

    fn poll_outcome(&mut self, cx: &mut outcome::Context) -> PollOutcome<Self::Output> {
        let item = try_poll_outcome!(self.outcome.poll_outcome(cx));
        cx.input().enter_scope(|| PollOutcome::Ready(item.map_err(Into::into)))
    }
}
