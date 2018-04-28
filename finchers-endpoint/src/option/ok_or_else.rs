use finchers_core::endpoint::{Context, Endpoint};
use finchers_core::outcome::{self, Outcome, PollOutcome};

#[derive(Debug, Copy, Clone)]
pub struct OkOrElse<E, F> {
    endpoint: E,
    f: F,
}

pub fn new<E, F, T, U>(endpoint: E, f: F) -> OkOrElse<E, F>
where
    E: Endpoint<Output = Option<T>>,
    F: FnOnce() -> U + Clone + Send,
{
    OkOrElse { endpoint, f }
}

impl<E, F, T, U> Endpoint for OkOrElse<E, F>
where
    E: Endpoint<Output = Option<T>>,
    F: FnOnce() -> U + Clone + Send,
{
    type Output = Result<T, U>;
    type Outcome = OkOrElseOutcome<E::Outcome, F>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Outcome> {
        Some(OkOrElseOutcome {
            outcome: self.endpoint.apply(cx)?,
            f: Some(self.f.clone()),
        })
    }
}

#[derive(Debug)]
pub struct OkOrElseOutcome<T, F> {
    outcome: T,
    f: Option<F>,
}

impl<T, F, A, U> Outcome for OkOrElseOutcome<T, F>
where
    T: Outcome<Output = Option<A>> + Send,
    F: FnOnce() -> U + Send,
{
    type Output = Result<A, U>;

    fn poll_outcome(&mut self, cx: &mut outcome::Context) -> PollOutcome<Self::Output> {
        let item = try_poll_outcome!(self.outcome.poll_outcome(cx));
        let f = self.f.take().expect("cannot resolve twice");
        cx.input().enter_scope(|| PollOutcome::Ready(item.ok_or_else(f)))
    }
}
