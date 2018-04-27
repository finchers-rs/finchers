use finchers_core::endpoint::{Context, Endpoint, IntoEndpoint};
use finchers_core::outcome::{self, Outcome, PollOutcome};

pub fn new<E, F>(endpoint: E, f: F) -> Inspect<E::Endpoint, F>
where
    E: IntoEndpoint,
    F: FnOnce(&E::Output) + Clone + Send,
{
    Inspect {
        endpoint: endpoint.into_endpoint(),
        f,
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Inspect<E, F> {
    endpoint: E,
    f: F,
}

impl<E, F> Endpoint for Inspect<E, F>
where
    E: Endpoint,
    F: FnOnce(&E::Output) + Clone + Send,
{
    type Output = E::Output;
    type Outcome = InspectOutcome<E::Outcome, F>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Outcome> {
        Some(InspectOutcome {
            outcome: self.endpoint.apply(cx)?,
            f: Some(self.f.clone()),
        })
    }
}

#[derive(Debug)]
pub struct InspectOutcome<T, F> {
    outcome: T,
    f: Option<F>,
}

impl<T, F> Outcome for InspectOutcome<T, F>
where
    T: Outcome + Send,
    F: FnOnce(&T::Output) + Send,
{
    type Output = T::Output;

    fn poll_outcome(&mut self, cx: &mut outcome::Context) -> PollOutcome<Self::Output> {
        let item = try_poll_outcome!(self.outcome.poll_outcome(cx));
        let f = self.f.take().expect("cannot resolve twice");
        cx.input().enter_scope(|| f(&item));
        PollOutcome::Ready(item)
    }
}
