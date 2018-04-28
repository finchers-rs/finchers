use finchers_core::endpoint::{Context, Endpoint};
use finchers_core::outcome::{self, Outcome, PollOutcome};

#[derive(Debug, Copy, Clone)]
pub struct MapSome<E, F> {
    endpoint: E,
    f: F,
}

pub fn new<E, F, U, T>(endpoint: E, f: F) -> MapSome<E, F>
where
    E: Endpoint<Output = Option<T>>,
    F: FnOnce(T) -> U + Clone + Send,
{
    MapSome { endpoint, f }
}

impl<E, F, T, U> Endpoint for MapSome<E, F>
where
    E: Endpoint<Output = Option<T>>,
    F: FnOnce(T) -> U + Clone + Send,
{
    type Output = Option<U>;
    type Outcome = MapSomeOutcome<E::Outcome, F>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Outcome> {
        Some(MapSomeOutcome {
            outcome: self.endpoint.apply(cx)?,
            f: Some(self.f.clone()),
        })
    }
}

#[derive(Debug)]
pub struct MapSomeOutcome<T, F> {
    outcome: T,
    f: Option<F>,
}

impl<T, F, A, U> Outcome for MapSomeOutcome<T, F>
where
    T: Outcome<Output = Option<A>> + Send,
    F: FnOnce(A) -> U + Send,
{
    type Output = Option<U>;

    fn poll_outcome(&mut self, cx: &mut outcome::Context) -> PollOutcome<Self::Output> {
        let item = try_poll_outcome!(self.outcome.poll_outcome(cx));
        let f = self.f.take().expect("cannot resolve twice");
        cx.input().enter_scope(|| PollOutcome::Ready(item.map(f)))
    }
}
