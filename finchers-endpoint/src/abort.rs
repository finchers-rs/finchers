use finchers_core::HttpError;
use finchers_core::endpoint::{Context, Endpoint};
use finchers_core::outcome;
use std::marker::PhantomData;

pub fn abort<F, T, E>(f: F) -> Abort<F, T>
where
    F: Fn(&mut Context) -> E,
    T: Send,
    E: HttpError,
{
    Abort {
        f,
        _marker: PhantomData,
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Abort<F, T> {
    f: F,
    _marker: PhantomData<fn() -> T>,
}

impl<F, T, E> Endpoint for Abort<F, T>
where
    F: Fn(&mut Context) -> E,
    T: Send,
    E: HttpError,
{
    type Output = T;
    type Outcome = outcome::Abort<T, E>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Outcome> {
        Some(outcome::abort((self.f)(cx)))
    }
}
