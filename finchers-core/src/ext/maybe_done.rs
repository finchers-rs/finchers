use self::MaybeDone::*;
use crate::poll::Poll;
use crate::task::Task;
use std::mem;

#[derive(Debug)]
pub enum MaybeDone<T: Task> {
    Pending(T),
    Done(T::Output),
    Gone,
}

impl<T: Task> MaybeDone<T> {
    pub fn poll_done(&mut self) -> bool {
        let item = match *self {
            Pending(ref mut f) => match f.poll_task() {
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
