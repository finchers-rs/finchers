use self::MaybeDone::*;
use crate::task::Task;
use crate::{Error, Poll};
use std::mem;

#[derive(Debug)]
pub enum MaybeDone<T: Task> {
    Pending(T),
    Done(T::Output),
    Gone,
}

impl<T: Task> MaybeDone<T> {
    pub fn poll_done(&mut self) -> Result<bool, Error> {
        let item = match *self {
            Pending(ref mut f) => match f.poll_task() {
                Poll::Ready(Ok(item)) => item,
                Poll::Pending => return Ok(false),
                Poll::Ready(Err(e)) => return Err(e),
            },
            Done(..) => return Ok(true),
            Gone => panic!("cannot join twice"),
        };
        *self = Done(item);
        Ok(true)
    }

    pub fn take_item(&mut self) -> T::Output {
        match mem::replace(self, Gone) {
            Done(item) => item,
            _ => panic!(),
        }
    }

    pub fn erase(&mut self) {
        *self = Gone;
    }
}
