use self::MaybeDone::*;
use crate::future::{Poll, TryFuture};
use std::{fmt, mem};

pub enum MaybeDone<T: TryFuture> {
    Pending(T),
    Done(T::Ok),
    Gone,
}

impl<T: TryFuture> fmt::Debug for MaybeDone<T>
where
    T: fmt::Debug,
    T::Ok: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MaybeDone::Pending(ref first) => f.debug_tuple("Pending").field(first).finish(),
            MaybeDone::Done(ref second) => f.debug_tuple("Done").field(second).finish(),
            MaybeDone::Gone => f.debug_tuple("Gone").finish(),
        }
    }
}

impl<T: TryFuture> MaybeDone<T> {
    pub fn poll_done(&mut self) -> Result<bool, T::Error> {
        let item = match *self {
            Pending(ref mut f) => match f.try_poll() {
                Poll::Ready(Ok(item)) => item,
                Poll::Ready(Err(err)) => return Err(err),
                Poll::Pending => return Ok(false),
            },
            Done(..) => return Ok(true),
            Gone => panic!("cannot join twice"),
        };
        *self = Done(item);
        Ok(true)
    }

    pub fn take_item(&mut self) -> T::Ok {
        match mem::replace(self, Gone) {
            Done(item) => item,
            _ => panic!(),
        }
    }
}
