// imported from futures::future::chain;

use std::mem;
use futures::{Async, Future, Poll};
use errors::HttpError;
use endpoint::EndpointError;

#[derive(Debug)]
pub enum Chain<A, B, C> {
    First(A, C),
    Second(B),
    Done,
}

use self::Chain::*;

impl<A, B, C, D> Chain<A, B, C>
where
    A: Future<Error = EndpointError<D>>,
    D: HttpError,
    B: Future,
{
    pub fn new(a: A, c: C) -> Self {
        Chain::First(a, c)
    }

    pub fn poll<F>(&mut self, f: F) -> Poll<B::Item, EndpointError<B::Error>>
    where
        F: FnOnce(Result<A::Item, D>, C) -> Result<Result<B::Item, B>, B::Error>,
        B::Error: HttpError,
    {
        let a_result = match *self {
            First(ref mut a, ..) => match a.poll() {
                Ok(Async::Ready(item)) => Ok(item),
                Ok(Async::NotReady) => return Ok(Async::NotReady),
                Err(EndpointError::Endpoint(err)) => Err(err),
                Err(EndpointError::Http(err)) => return Err(err.into()),
            },
            Second(ref mut b) => return b.poll().map_err(Into::into),
            Done => panic!("cannot poll twice"),
        };

        let data = match mem::replace(self, Done) {
            First(_, c) => c,
            _ => panic!(),
        };

        match f(a_result, data)? {
            Ok(item) => Ok(Async::Ready(item)),
            Err(mut b) => {
                let result = b.poll().map_err(Into::into);
                *self = Second(b);
                result
            }
        }
    }
}
