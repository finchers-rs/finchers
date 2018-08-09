#![allow(missing_docs)]

use crate::either::Either;
use crate::endpoint::{Context, EndpointBase};
use crate::future::{Future, Poll, TryFuture};
use crate::generic::Tuple;
use std::{fmt, mem};

#[derive(Debug, Copy, Clone)]
pub struct OrElse<E, F> {
    pub(super) endpoint: E,
    pub(super) f: F,
}

impl<E, F, R> EndpointBase for OrElse<E, F>
where
    E: EndpointBase,
    F: FnOnce(E::Error) -> R + Clone,
    R: TryFuture<Ok = E::Ok>,
{
    type Ok = R::Ok;
    type Error = R::Error;
    type Future = OrElseFuture<E::Future, F, R>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Future> {
        Some(OrElseFuture {
            state: State::First(self.endpoint.apply(cx)?, self.f.clone()),
        })
    }
}

pub struct OrElseFuture<T, F, R>
where
    T: TryFuture,
    T::Ok: Tuple,
    F: FnOnce(T::Error) -> R,
    R: TryFuture<Ok = T::Ok>,
{
    state: State<T, F, R>,
}

impl<T, F, R> fmt::Debug for OrElseFuture<T, F, R>
where
    T: TryFuture + fmt::Debug,
    T::Ok: Tuple,
    F: FnOnce(T::Error) -> R + fmt::Debug,
    R: TryFuture<Ok = T::Ok> + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("OrElseFuture")
            .field("state", &self.state)
            .finish()
    }
}

enum State<T, F, R>
where
    T: TryFuture,
    T::Ok: Tuple,
    F: FnOnce(T::Error) -> R,
    R: TryFuture<Ok = T::Ok>,
{
    First(T, F),
    Second(R),
    Done,
}

impl<T, F, R> fmt::Debug for State<T, F, R>
where
    T: TryFuture + fmt::Debug,
    T::Ok: Tuple,
    F: FnOnce(T::Error) -> R + fmt::Debug,
    R: TryFuture<Ok = T::Ok> + fmt::Debug,
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

impl<T, F, R> Future for OrElseFuture<T, F, R>
where
    T: TryFuture,
    T::Ok: Tuple,
    F: FnOnce(T::Error) -> R,
    R: TryFuture<Ok = T::Ok>,
{
    type Output = Result<R::Ok, R::Error>;

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
                (State::First(..), Either::Left(Ok(x))) => return Poll::Ready(Ok(x)),
                (State::First(_, f), Either::Left(Err(e))) => self.state = State::Second(f(e)),
                (State::Second(..), Either::Right(out)) => return Poll::Ready(out),
                _ => unreachable!("unexpected condition"),
            }
        }
    }
}
