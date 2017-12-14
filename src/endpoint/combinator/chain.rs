// imported from futures::future::chain;

use std::mem;
use futures::{Async, Future, Poll};

#[derive(Debug)]
pub enum Chain<A, B, C> {
    First(A, C),
    Second(B),
    Done,
}

use self::Chain::*;

impl<A: Future, B: Future, C> Chain<A, B, C> {
    pub fn new(a: A, c: C) -> Self {
        Chain::First(a, c)
    }

    pub fn poll<F>(&mut self, f: F) -> Poll<B::Item, B::Error>
    where
        F: FnOnce(Result<A::Item, A::Error>, C) -> Result<Result<B::Item, B>, B::Error>,
    {
        let a_result = match *self {
            First(ref mut a, ..) => match a.poll() {
                Ok(Async::Ready(item)) => Ok(item),
                Ok(Async::NotReady) => return Ok(Async::NotReady),
                Err(err) => Err(err),
            },
            Second(ref mut b) => return b.poll(),
            Done => panic!("cannot poll twice"),
        };

        let data = match mem::replace(self, Done) {
            First(_, c) => c,
            _ => panic!(),
        };

        match f(a_result, data)? {
            Ok(item) => Ok(Async::Ready(item)),
            Err(mut b) => {
                let result = b.poll();
                *self = Second(b);
                result
            }
        }
    }
}
