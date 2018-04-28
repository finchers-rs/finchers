use finchers_core::HttpError;
use finchers_core::endpoint::{Context, Endpoint};
use finchers_core::outcome::{self, Outcome, PollOutcome};

pub fn new<E, T, R>(endpoint: E) -> UnwrapOk<E>
where
    E: Endpoint<Output = Result<T, R>>,
    R: HttpError,
{
    UnwrapOk { endpoint }
}

#[derive(Copy, Clone, Debug)]
pub struct UnwrapOk<E> {
    endpoint: E,
}

impl<E, T, R> Endpoint for UnwrapOk<E>
where
    E: Endpoint<Output = Result<T, R>>,
    R: HttpError,
{
    type Output = T;
    type Outcome = UnwrapOkOutcome<E::Outcome>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Outcome> {
        Some(UnwrapOkOutcome {
            outcome: self.endpoint.apply(cx)?,
        })
    }
}

#[derive(Debug)]
pub struct UnwrapOkOutcome<T> {
    outcome: T,
}

impl<T, U, E> Outcome for UnwrapOkOutcome<T>
where
    T: Outcome<Output = Result<U, E>> + Send,
    E: HttpError,
{
    type Output = U;

    fn poll_outcome(&mut self, cx: &mut outcome::Context) -> PollOutcome<Self::Output> {
        match try_poll_outcome!(self.outcome.poll_outcome(cx)) {
            Ok(item) => PollOutcome::Ready(item),
            Err(err) => PollOutcome::Abort(Into::into(err)),
        }
    }
}
