use self::MaybeDone::*;
use finchers_core::Error;
use finchers_core::outcome::{self, Outcome, PollOutcome};
use std::mem;

pub enum MaybeDone<T: Outcome> {
    Pending(T),
    Done(T::Output),
    Gone,
}

impl<T: Outcome> MaybeDone<T> {
    pub fn poll_done(&mut self, cx: &mut outcome::Context) -> Result<bool, Error> {
        let item = match *self {
            Pending(ref mut f) => match f.poll_outcome(cx) {
                PollOutcome::Ready(item) => item,
                PollOutcome::Pending => return Ok(false),
                PollOutcome::Abort(e) => return Err(e),
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
