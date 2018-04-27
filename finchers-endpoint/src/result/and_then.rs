use finchers_core::endpoint::{Context, Endpoint};
use finchers_core::outcome::{self, Outcome, PollOutcome};

#[derive(Debug, Copy, Clone)]
pub struct AndThen<E, F> {
    endpoint: E,
    f: F,
}

pub fn new<E, F, U, A, B>(endpoint: E, f: F) -> AndThen<E, F>
where
    E: Endpoint<Output = Result<A, B>>,
    F: FnOnce(A) -> Result<U, B> + Clone + Send,
{
    AndThen { endpoint, f }
}

impl<E, F, A, B, U> Endpoint for AndThen<E, F>
where
    E: Endpoint<Output = Result<A, B>>,
    F: FnOnce(A) -> Result<U, B> + Clone + Send,
{
    type Output = Result<U, B>;
    type Outcome = AndThenOutcome<E::Outcome, F>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Outcome> {
        Some(AndThenOutcome {
            outcome: self.endpoint.apply(cx)?,
            f: Some(self.f.clone()),
        })
    }
}

#[derive(Debug)]
pub struct AndThenOutcome<T, F> {
    outcome: T,
    f: Option<F>,
}

impl<T, F, U, A, B> Outcome for AndThenOutcome<T, F>
where
    T: Outcome<Output = Result<A, B>> + Send,
    F: FnOnce(A) -> Result<U, B> + Send,
{
    type Output = Result<U, B>;

    fn poll_outcome(&mut self, cx: &mut outcome::Context) -> PollOutcome<Self::Output> {
        let item = try_poll_outcome!(self.outcome.poll_outcome(cx));
        let f = self.f.take().expect("cannot resolve twice");
        cx.input().enter_scope(|| PollOutcome::Ready(item.and_then(f)))
    }
}
