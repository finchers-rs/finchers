use self::MaybeDone::*;
use finchers_core::endpoint::{Error, task::{self, Async, Future}};
use std::mem;

pub enum MaybeDone<F: Future> {
    Pending(F),
    Done(F::Item),
    Gone,
}

impl<F: Future> MaybeDone<F> {
    pub fn poll_done(&mut self, cx: &mut task::Context) -> Result<bool, Error> {
        let item = match *self {
            Pending(ref mut f) => match f.poll(cx)? {
                Async::Ready(item) => item,
                Async::NotReady => return Ok(false),
            },
            Done(..) => return Ok(true),
            Gone => panic!("cannot join twice"),
        };
        *self = Done(item);
        Ok(true)
    }

    pub fn take_item(&mut self) -> F::Item {
        match mem::replace(self, Gone) {
            Done(item) => item,
            _ => panic!(),
        }
    }

    pub fn erase(&mut self) {
        *self = Gone;
    }
}
