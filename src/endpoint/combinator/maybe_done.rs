use std::mem;
use futures::{Async, Future};

#[derive(Debug)]
pub enum MaybeDone<A: Future> {
    NotYet(A),
    Done(A::Item),
    Gone,
}

use self::MaybeDone::*;

impl<A: Future> MaybeDone<A> {
    pub fn poll(&mut self) -> Result<bool, A::Error> {
        let result = match *self {
            NotYet(ref mut a) => a.poll()?,
            Done(..) => return Ok(true),
            Gone => panic!("cannot resolve twice"),
        };

        match result {
            Async::Ready(result) => {
                *self = Done(result);
                Ok(true)
            }
            Async::NotReady => Ok(false),
        }
    }

    pub fn take(&mut self) -> A::Item {
        match mem::replace(self, Gone) {
            Done(a) => a,
            _ => panic!(),
        }
    }
}
