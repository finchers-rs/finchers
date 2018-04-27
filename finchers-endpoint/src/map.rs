use finchers_core::endpoint::{Context, Endpoint, IntoEndpoint};
use finchers_core::outcome::{self, Outcome, PollOutcome};

pub fn new<E, F, T>(endpoint: E, f: F) -> Map<E::Endpoint, F>
where
    E: IntoEndpoint,
    F: FnOnce(E::Output) -> T + Clone + Send,
{
    Map {
        endpoint: endpoint.into_endpoint(),
        f,
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Map<E, F> {
    endpoint: E,
    f: F,
}

impl<E, F, T> Endpoint for Map<E, F>
where
    E: Endpoint,
    F: FnOnce(E::Output) -> T + Clone + Send,
{
    type Output = F::Output;
    type Outcome = MapOutcome<E::Outcome, F>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Outcome> {
        Some(MapOutcome {
            outcome: self.endpoint.apply(cx)?,
            f: Some(self.f.clone()),
        })
    }
}

#[derive(Debug)]
pub struct MapOutcome<T, F> {
    outcome: T,
    f: Option<F>,
}

impl<T, F, U> Outcome for MapOutcome<T, F>
where
    T: Outcome + Send,
    F: FnOnce(T::Output) -> U + Send,
{
    type Output = U;

    fn poll_outcome(&mut self, cx: &mut outcome::Context) -> PollOutcome<Self::Output> {
        let item = try_poll_outcome!(self.outcome.poll_outcome(cx));
        let f = self.f.take().expect("cannot resolve twice");
        cx.input().enter_scope(|| PollOutcome::Ready(f(item)))
    }
}
