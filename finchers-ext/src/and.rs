#![allow(missing_docs)]

use super::maybe_done::MaybeDone;
use finchers_core::endpoint::{Context, Endpoint, IntoEndpoint};
use finchers_core::task::{self, Task};
use finchers_core::{Error, Poll, PollResult};
use std::fmt;

pub fn new<E1, E2>(e1: E1, e2: E2) -> And<E1::Endpoint, E2::Endpoint>
where
    E1: IntoEndpoint,
    E2: IntoEndpoint,
    E1::Output: Send,
    E2::Output: Send,
{
    And {
        e1: e1.into_endpoint(),
        e2: e2.into_endpoint(),
    }
}

#[derive(Copy, Clone, Debug)]
pub struct And<E1, E2> {
    e1: E1,
    e2: E2,
}

impl<E1, E2> Endpoint for And<E1, E2>
where
    E1: Endpoint,
    E2: Endpoint,
    E1::Output: Send,
    E2::Output: Send,
{
    type Output = (E1::Output, E2::Output);
    type Task = AndTask<E1::Task, E2::Task>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Task> {
        let f1 = self.e1.apply(cx)?;
        let f2 = self.e2.apply(cx)?;
        Some(AndTask {
            f1: MaybeDone::Pending(f1),
            f2: MaybeDone::Pending(f2),
        })
    }
}

pub struct AndTask<F1: Task, F2: Task> {
    f1: MaybeDone<F1>,
    f2: MaybeDone<F2>,
}

impl<T1, T2> fmt::Debug for AndTask<T1, T2>
where
    T1: Task + fmt::Debug,
    T2: Task + fmt::Debug,
    T1::Output: fmt::Debug,
    T2::Output: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("AndTask")
            .field("t1", &self.f1)
            .field("t2", &self.f2)
            .finish()
    }
}

impl<F1, F2> Task for AndTask<F1, F2>
where
    F1: Task,
    F2: Task,
    F1::Output: Send,
    F2::Output: Send,
{
    type Output = (F1::Output, F2::Output);

    fn poll_task(&mut self, cx: &mut task::Context) -> PollResult<Self::Output, Error> {
        let mut all_done = match self.f1.poll_done(cx) {
            Ok(done) => done,
            Err(e) => {
                self.f1.erase();
                self.f2.erase();
                return Poll::Ready(Err(e));
            }
        };
        all_done = match self.f2.poll_done(cx) {
            Ok(done) => all_done && done,
            Err(e) => {
                self.f1.erase();
                self.f2.erase();
                return Poll::Ready(Err(e));
            }
        };

        if all_done {
            Poll::Ready(Ok((self.f1.take_item(), self.f2.take_item())))
        } else {
            Poll::Pending
        }
    }
}
