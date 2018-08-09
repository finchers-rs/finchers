#![allow(missing_docs)]

use crate::either::Either;
use crate::endpoint::{Context, EndpointBase};
use crate::future::{Future, Poll, TryFuture};
use crate::generic::{Func, Tuple};
use std::{fmt, mem};

#[derive(Debug, Copy, Clone)]
pub struct AndThen<E, F> {
    pub(super) endpoint: E,
    pub(super) f: F,
}

impl<E, F> EndpointBase for AndThen<E, F>
where
    E: EndpointBase,
    F: Func<E::Ok> + Clone,
    F::Out: TryFuture<Error = E::Error>,
    <F::Out as TryFuture>::Ok: Tuple,
{
    type Ok = <F::Out as TryFuture>::Ok;
    type Error = E::Error;
    type Future = AndThenFuture<E::Future, F>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Future> {
        Some(AndThenFuture {
            state: State::First(self.endpoint.apply(cx)?, self.f.clone()),
        })
    }
}

pub struct AndThenFuture<T, F>
where
    T: TryFuture,
    T::Ok: Tuple,
    F: Func<T::Ok>,
    F::Out: TryFuture<Error = T::Error>,
    <F::Out as TryFuture>::Ok: Tuple,
{
    state: State<T, F>,
}

impl<T, F> fmt::Debug for AndThenFuture<T, F>
where
    T: TryFuture + fmt::Debug,
    T::Ok: Tuple,
    F: Func<T::Ok> + fmt::Debug,
    F::Out: TryFuture<Error = T::Error> + fmt::Debug,
    <F::Out as TryFuture>::Ok: Tuple,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("AndThenFuture")
            .field("state", &self.state)
            .finish()
    }
}

enum State<T, F>
where
    T: TryFuture,
    T::Ok: Tuple,
    F: Func<T::Ok>,
    F::Out: TryFuture<Error = T::Error>,
    <F::Out as TryFuture>::Ok: Tuple,
{
    First(T, F),
    Second(F::Out),
    Done,
}

impl<T, F> fmt::Debug for State<T, F>
where
    T: TryFuture + fmt::Debug,
    T::Ok: Tuple,
    F: Func<T::Ok> + fmt::Debug,
    F::Out: TryFuture<Error = T::Error> + fmt::Debug,
    <F::Out as TryFuture>::Ok: Tuple,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            State::First(ref first, ref f) => formatter
                .debug_tuple("First")
                .field(first)
                .field(f)
                .finish(),
            State::Second(ref second) => formatter.debug_tuple("Second").field(second).finish(),
            State::Done => formatter.debug_tuple("Done").finish(),
        }
    }
}

impl<T, F> Future for AndThenFuture<T, F>
where
    T: TryFuture,
    T::Ok: Tuple,
    F: Func<T::Ok>,
    F::Out: TryFuture<Error = T::Error>,
    <F::Out as TryFuture>::Ok: Tuple,
{
    type Output = Result<<F::Out as TryFuture>::Ok, T::Error>;

    fn poll(&mut self) -> Poll<Self::Output> {
        loop {
            // 1. poll the internal state.
            let polled = match self.state {
                State::First(ref mut future, ..) => Either::Left(poll!(future.try_poll())),
                State::Second(ref mut future) => Either::Right(poll!(future.try_poll())),
                State::Done => panic!("This future has already polled."),
            };

            // 2. transit to the next state.
            match (mem::replace(&mut self.state, State::Done), polled) {
                (State::First(_, f), Either::Left(Ok(x))) => self.state = State::Second(f.call(x)),
                (State::First(..), Either::Left(Err(e))) => return Poll::Ready(Err(e)),
                (State::Second(..), Either::Right(out)) => return Poll::Ready(out),
                _ => unreachable!("unexpected condition"),
            }
        }
    }
}
