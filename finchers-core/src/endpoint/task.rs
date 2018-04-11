use super::Error;
use Input;
use futures;

pub use futures::Async;

pub type Poll<T> = Result<Async<T>, Error>;

pub struct Context<'a> {
    pub input: &'a mut Input,
}

pub trait Future {
    type Item;

    fn poll(&mut self, cx: &mut Context) -> Poll<Self::Item>;
}

impl<F> Future for F
where
    F: futures::Future,
    F::Error: Into<Error>,
{
    type Item = F::Item;

    fn poll(&mut self, _: &mut Context) -> Poll<Self::Item> {
        futures::Future::poll(self).map_err(Into::into)
    }
}
