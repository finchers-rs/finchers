use super::maybe_done::MaybeDone;
use finchers_core::endpoint::{Context, Endpoint, IntoEndpoint};
use finchers_core::outcome::{self, Outcome, PollOutcome};

pub fn new<E1, E2>(e1: E1, e2: E2) -> And<E1::Endpoint, E2::Endpoint>
where
    E1: IntoEndpoint,
    E2: IntoEndpoint,
    E1::Output: Send,
    E2::Output: Send,
{
    And {
        e1: e1.into_endpoint(),
        e2: e2.into_endpoint(),
    }
}

#[derive(Copy, Clone, Debug)]
pub struct And<E1, E2> {
    e1: E1,
    e2: E2,
}

impl<E1, E2> Endpoint for And<E1, E2>
where
    E1: Endpoint,
    E2: Endpoint,
    E1::Output: Send,
    E2::Output: Send,
{
    type Output = (E1::Output, E2::Output);
    type Outcome = AndOutcome<E1::Outcome, E2::Outcome>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Outcome> {
        let f1 = self.e1.apply(cx)?;
        let f2 = self.e2.apply(cx)?;
        Some(AndOutcome {
            f1: MaybeDone::Pending(f1),
            f2: MaybeDone::Pending(f2),
        })
    }
}

pub struct AndOutcome<F1: Outcome, F2: Outcome> {
    f1: MaybeDone<F1>,
    f2: MaybeDone<F2>,
}

impl<F1, F2> Outcome for AndOutcome<F1, F2>
where
    F1: Outcome,
    F2: Outcome,
{
    type Output = (F1::Output, F2::Output);

    fn poll_outcome(&mut self, cx: &mut outcome::Context) -> PollOutcome<Self::Output> {
        let mut all_done = match self.f1.poll_done(cx) {
            Ok(done) => done,
            Err(e) => {
                self.f1.erase();
                self.f2.erase();
                return PollOutcome::Abort(e);
            }
        };
        all_done = match self.f2.poll_done(cx) {
            Ok(done) => all_done && done,
            Err(e) => {
                self.f1.erase();
                self.f2.erase();
                return PollOutcome::Abort(e);
            }
        };

        if all_done {
            PollOutcome::Ready((self.f1.take_item(), self.f2.take_item()))
        } else {
            PollOutcome::Pending
        }
    }
}
