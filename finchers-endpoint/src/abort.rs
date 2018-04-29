use finchers_core::endpoint::{Context, Endpoint};
use finchers_core::{task, HttpError, Never};

pub fn abort<F, E>(f: F) -> Abort<F>
where
    F: Fn(&mut Context) -> E,
    E: HttpError,
{
    Abort { f }
}

#[derive(Copy, Clone, Debug)]
pub struct Abort<F> {
    f: F,
}

impl<F, E> Endpoint for Abort<F>
where
    F: Fn(&mut Context) -> E,
    E: HttpError,
{
    type Output = Never;
    type Task = task::Abort<E>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Task> {
        Some(task::abort((self.f)(cx)))
    }
}
