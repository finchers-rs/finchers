#![allow(missing_docs)]

use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::Arc;
use futures::{Future, IntoFuture};

pub trait Process {
    type In;
    type InErr;
    type Out;
    type OutErr;
    type Future: Future<Item = Self::Out, Error = Self::OutErr>;

    fn call(&self, input: Option<Result<Self::In, Self::InErr>>) -> Self::Future;
}

impl<P: Process> Process for Rc<P> {
    type In = P::In;
    type InErr = P::InErr;
    type Out = P::Out;
    type OutErr = P::OutErr;
    type Future = P::Future;

    fn call(&self, input: Option<Result<Self::In, Self::InErr>>) -> Self::Future {
        (**self).call(input)
    }
}

impl<P: Process> Process for Arc<P> {
    type In = P::In;
    type InErr = P::InErr;
    type Out = P::Out;
    type OutErr = P::OutErr;
    type Future = P::Future;

    fn call(&self, input: Option<Result<Self::In, Self::InErr>>) -> Self::Future {
        (**self).call(input)
    }
}

#[allow(missing_docs)]
pub fn process_fn<F, A, B, R>(f: F) -> ProcessFn<F, A, B, R>
where
    F: Fn(Option<Result<A, B>>) -> R,
    R: IntoFuture,
{
    ProcessFn {
        f,
        _marker: PhantomData,
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct ProcessFn<F, A, B, R>
where
    F: Fn(Option<Result<A, B>>) -> R,
    R: IntoFuture,
{
    f: F,
    _marker: PhantomData<fn((A, B)) -> R>,
}

impl<F, A, B, R> Process for ProcessFn<F, A, B, R>
where
    F: Fn(Option<Result<A, B>>) -> R,
    R: IntoFuture,
{
    type In = A;
    type InErr = B;
    type Out = R::Item;
    type OutErr = R::Error;
    type Future = R::Future;

    fn call(&self, input: Option<Result<Self::In, Self::InErr>>) -> Self::Future {
        (self.f)(input).into_future()
    }
}
