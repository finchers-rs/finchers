use self::MaybeDone::*;
use finchers_core::Error;
use finchers_core::task::{self, Async, Task};
use std::mem;

pub enum MaybeDone<T: Task> {
    Pending(T),
    Done(T::Output),
    Gone,
}

impl<T: Task> MaybeDone<T> {
    pub fn poll_done(&mut self, cx: &mut task::Context) -> Result<bool, Error> {
        let item = match *self {
            Pending(ref mut f) => match f.poll_task(cx)? {
                Async::Ready(item) => item,
                Async::NotReady => return Ok(false),
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
