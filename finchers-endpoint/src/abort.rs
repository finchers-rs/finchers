use finchers_core::endpoint::{Context, Endpoint};
use finchers_core::task::CompatTask;
use finchers_core::{Error, HttpError};
use futures::future::{err, FutureResult};
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
    type Item = T;
    type Task = CompatTask<FutureResult<T, Error>>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Task> {
        Some(CompatTask::from(err(Error::from((self.f)(cx)))))
    }
}
