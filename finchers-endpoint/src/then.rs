use finchers_core::HttpError;
use finchers_core::endpoint::{Context, Endpoint};
use finchers_core::outcome::{self, Outcome, PollOutcome};
use futures::{Async, Future, IntoFuture};
use std::mem;

pub fn new<E, F, R>(endpoint: E, f: F) -> Then<E, F>
where
    E: Endpoint,
    F: FnOnce(E::Output) -> R + Clone + Send,
    R: IntoFuture,
    R::Future: Send,
    R::Error: HttpError,
{
    Then { endpoint, f }
}

#[derive(Copy, Clone, Debug)]
pub struct Then<E, F> {
    endpoint: E,
    f: F,
}

impl<E, F, R> Endpoint for Then<E, F>
where
    E: Endpoint,
    F: FnOnce(E::Output) -> R + Clone + Send,
    R: IntoFuture,
    R::Future: Send,
    R::Error: HttpError,
{
    type Output = R::Item;
    type Outcome = ThenOutcome<E::Outcome, F, R>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Outcome> {
        let outcome = self.endpoint.apply(cx)?;
        Some(ThenOutcome::First(outcome, self.f.clone()))
    }
}

#[derive(Debug)]
pub enum ThenOutcome<T, F, R>
where
    T: Outcome,
    F: FnOnce(T::Output) -> R + Send,
    R: IntoFuture,
    R::Future: Send,
    R::Error: HttpError,
{
    First(T, F),
    Second(R::Future),
    Done,
}

impl<T, F, R> Outcome for ThenOutcome<T, F, R>
where
    T: Outcome,
    F: FnOnce(T::Output) -> R + Send,
    R: IntoFuture,
    R::Future: Send,
    R::Error: HttpError,
{
    type Output = R::Item;

    fn poll_outcome(&mut self, cx: &mut outcome::Context) -> PollOutcome<Self::Output> {
        use self::ThenOutcome::*;
        loop {
            // TODO: optimize
            match mem::replace(self, Done) {
                First(mut outcome, f) => match outcome.poll_outcome(cx) {
                    PollOutcome::Pending => {
                        *self = First(outcome, f);
                        return PollOutcome::Pending;
                    }
                    PollOutcome::Ready(r) => {
                        cx.input().enter_scope(|| {
                            *self = Second(f(r).into_future());
                        });
                        continue;
                    }
                    PollOutcome::Abort(e) => return PollOutcome::Abort(e),
                },
                Second(mut fut) => {
                    return match fut.poll() {
                        Ok(Async::NotReady) => {
                            *self = Second(fut);
                            PollOutcome::Pending
                        }
                        Ok(Async::Ready(item)) => PollOutcome::Ready(item),
                        Err(err) => PollOutcome::Abort(Into::into(err)),
                    }
                }
                Done => panic!(),
            }
        }
    }
}
