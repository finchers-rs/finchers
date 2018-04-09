use self::Chain::*;
use Error;
use finchers_core::HttpError;
use futures::Async::*;
use futures::{Future, Poll};
use std::mem;

#[derive(Debug)]
pub enum Chain<A, B, C> {
    First(A, C),
    Second(B),
    Done,
}

impl<A, B, C> Chain<A, B, C>
where
    A: Future<Error = Error>,
    B: Future,
    B::Error: HttpError,
{
    pub fn new(a: A, c: C) -> Self {
        Chain::First(a, c)
    }

    pub fn poll<F>(&mut self, f: F) -> Poll<B::Item, Error>
    where
        F: FnOnce(Result<A::Item, Error>, C) -> Result<Result<B::Item, B>, Error>,
    {
        let a_result = match *self {
            First(ref mut a, ..) => match a.poll() {
                Ok(Ready(item)) => Ok(item),
                Ok(NotReady) => return Ok(NotReady),
                Err(e) => Err(e),
            },
            Second(ref mut b) => return b.poll().map_err(Into::into),
            Done => panic!("cannot poll twice"),
        };

        let data = match mem::replace(self, Done) {
            First(_, c) => c,
            _ => panic!(),
        };

        match f(a_result, data)? {
            Ok(item) => Ok(Ready(item)),
            Err(mut b) => {
                let result = b.poll().map_err(Into::into);
                *self = Second(b);
                result
            }
        }
    }
}
