// imported from futures::future::chain;

use std::mem;
use futures::{Async, Future, Poll};
use http::Error;

#[derive(Debug)]
pub enum Chain<A, B, C> {
    First(A, C),
    Second(B),
    Done,
}

use self::Chain::*;

impl<A, B, C, D> Chain<A, B, C>
where
    A: Future<Error = Result<D, Error>>,
    B: Future,
{
    pub fn new(a: A, c: C) -> Self {
        Chain::First(a, c)
    }

    pub fn poll<F>(&mut self, f: F) -> Poll<B::Item, Result<B::Error, Error>>
    where
        F: FnOnce(Result<A::Item, D>, C) -> Result<Result<B::Item, B>, B::Error>,
    {
        let a_result = match *self {
            First(ref mut a, ..) => match a.poll() {
                Ok(Async::Ready(item)) => Ok(item),
                Ok(Async::NotReady) => return Ok(Async::NotReady),
                Err(Ok(err)) => Err(err),
                Err(Err(err)) => return Err(Err(err)),
            },
            Second(ref mut b) => return b.poll().map_err(Ok),
            Done => panic!("cannot poll twice"),
        };

        let data = match mem::replace(self, Done) {
            First(_, c) => c,
            _ => panic!(),
        };

        match f(a_result, data).map_err(Ok)? {
            Ok(item) => Ok(Async::Ready(item)),
            Err(mut b) => {
                let result = b.poll().map_err(Ok);
                *self = Second(b);
                result
            }
        }
    }
}
