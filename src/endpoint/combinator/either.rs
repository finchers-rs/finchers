#![allow(missing_docs)]

//! Definition of `Either`s

use std::fmt::{self, Display};
use std::error::Error;
use futures::{Async, Future, Poll};
use response::{Responder, Response};


#[derive(Debug)]
pub enum Either2<E1, E2> {
    E1(E1),
    E2(E2),
}

impl<E1: Display, E2: Display> Display for Either2<E1, E2> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Either2::E1(ref e) => write!(f, "Either2::E1({})", e),
            Either2::E2(ref e) => write!(f, "Either2::E2({})", e),
        }
    }
}

impl<E1: Error, E2: Error> Error for Either2<E1, E2> {
    fn description(&self) -> &str {
        match *self {
            Either2::E1(ref e) => e.description(),
            Either2::E2(ref e) => e.description(),
        }
    }
}

impl<E1, E2> Future for Either2<E1, E2>
where
    E1: Future,
    E2: Future<Error = E1::Error>,
{
    type Item = Either2<E1::Item, E2::Item>;
    type Error = E1::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match *self {
            Either2::E1(ref mut e) => Ok(Async::Ready(Either2::E1(try_ready!(e.poll())))),
            Either2::E2(ref mut e) => Ok(Async::Ready(Either2::E2(try_ready!(e.poll())))),
        }
    }
}

impl<E1, E2> Responder for Either2<E1, E2>
where
    E1: Responder,
    E2: Responder,
{
    type Error = Either2<E1::Error, E2::Error>;

    fn respond(self) -> Result<Response, Self::Error> {
        match self {
            Either2::E1(e) => e.respond().map_err(Either2::E1),
            Either2::E2(e) => e.respond().map_err(Either2::E2),
        }
    }
}
