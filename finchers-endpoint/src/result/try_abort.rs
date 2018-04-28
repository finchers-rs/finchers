use finchers_core::HttpError;
use finchers_core::endpoint::{Context, Endpoint};
use finchers_core::outcome::{self, Outcome, PollOutcome};

pub fn new<E, T, R>(endpoint: E) -> TryAbort<E>
where
    E: Endpoint<Output = Result<T, R>>,
    R: HttpError,
{
    TryAbort { endpoint }
}

#[derive(Copy, Clone, Debug)]
pub struct TryAbort<E> {
    endpoint: E,
}

impl<E, T, R> Endpoint for TryAbort<E>
where
    E: Endpoint<Output = Result<T, R>>,
    R: HttpError,
{
    type Output = T;
    type Outcome = TryAbortOutcome<E::Outcome>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Outcome> {
        Some(TryAbortOutcome {
            outcome: self.endpoint.apply(cx)?,
        })
    }
}

#[derive(Debug)]
pub struct TryAbortOutcome<T> {
    outcome: T,
}

impl<T, U, E> Outcome for TryAbortOutcome<T>
where
    T: Outcome<Output = Result<U, E>> + Send,
    E: HttpError,
{
    type Output = U;

    fn poll_outcome(&mut self, cx: &mut outcome::Context) -> PollOutcome<Self::Output> {
        let item = try_poll_outcome!(self.outcome.poll_outcome(cx));
        cx.input().enter_scope(|| match item {
            Ok(item) => PollOutcome::Ready(item),
            Err(err) => PollOutcome::Abort(Into::into(err)),
        })
    }
}
