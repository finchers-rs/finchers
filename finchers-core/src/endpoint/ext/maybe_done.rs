use self::MaybeDone::*;
use crate::future::{Future, Poll};
use std::mem;

#[derive(Debug)]
pub enum MaybeDone<T: Future> {
    Pending(T),
    Done(T::Output),
    Gone,
}

impl<T: Future> MaybeDone<T> {
    pub fn poll_done(&mut self) -> bool {
        let item = match *self {
            Pending(ref mut f) => match f.poll() {
                Poll::Ready(item) => item,
                Poll::Pending => return false,
            },
            Done(..) => return true,
            Gone => panic!("cannot join twice"),
        };
        *self = Done(item);
        true
    }

    pub fn take_item(&mut self) -> T::Output {
        match mem::replace(self, Gone) {
            Done(item) => item,
            _ => panic!(),
        }
    }
}
