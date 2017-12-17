use std::mem;
use task::{Async, Task, TaskContext};

#[derive(Debug)]
pub enum MaybeDone<A: Task> {
    NotYet(A),
    Done(A::Item),
    Gone,
}

use self::MaybeDone::*;

impl<A: Task> MaybeDone<A> {
    pub fn poll(&mut self, ctx: &mut TaskContext) -> Result<bool, A::Error> {
        let result = match *self {
            NotYet(ref mut a) => a.poll(ctx)?,
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
