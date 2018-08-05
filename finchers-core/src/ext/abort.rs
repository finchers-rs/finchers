use crate::endpoint::{Context, Endpoint};
use crate::{task, Error, Never};

/// Create an endpoint which always abort the incoming request with an error of `E`.
pub fn abort<F, E>(f: F) -> Abort<F>
where
    F: Fn(&mut Context) -> E + Send + Sync,
    E: Into<Error> + Send,
{
    Abort { f }
}

#[allow(missing_docs)]
#[derive(Copy, Clone, Debug)]
pub struct Abort<F> {
    f: F,
}

impl<F, E> Endpoint for Abort<F>
where
    F: Fn(&mut Context) -> E + Send + Sync,
    E: Into<Error> + Send,
{
    type Output = Never;
    type Task = task::Abort<E>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Task> {
        Some(task::abort((self.f)(cx)))
    }
}
