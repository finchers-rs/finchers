use crate::endpoint::{Context, EndpointBase};
use crate::{task, Error, Never};

/// Create an endpoint which always abort the incoming request with an error of `E`.
pub fn abort<F, E>(f: F) -> Abort<F>
where
    F: Fn(&mut Context) -> E,
    E: Into<Error>,
{
    Abort { f }
}

#[allow(missing_docs)]
#[derive(Copy, Clone, Debug)]
pub struct Abort<F> {
    f: F,
}

impl<F, E> EndpointBase for Abort<F>
where
    F: Fn(&mut Context) -> E,
    E: Into<Error>,
{
    type Output = Never;
    type Task = task::Abort<E>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Task> {
        Some(task::abort((self.f)(cx)))
    }
}
