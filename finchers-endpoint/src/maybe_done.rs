use self::MaybeDone::*;
use finchers_core::Error;
use finchers_core::task::{self, PollTask, Task};
use std::mem;

pub enum MaybeDone<T: Task> {
    Pending(T),
    Done(T::Output),
    Gone,
}

impl<T: Task> MaybeDone<T> {
    pub fn poll_done(&mut self, cx: &mut task::Context) -> Result<bool, Error> {
        let item = match *self {
            Pending(ref mut f) => match f.poll_task(cx) {
                PollTask::Ready(item) => item,
                PollTask::Pending => return Ok(false),
                PollTask::Aborted(e) => return Err(e),
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
