use finchers_core::endpoint::{Context, Endpoint};
use finchers_core::task::{self, IntoTask, PollTask, Task};
use std::mem;

pub fn new<E, F, R>(endpoint: E, f: F) -> Then<E, F>
where
    E: Endpoint,
    F: FnOnce(E::Output) -> R + Clone + Send,
    R: IntoTask,
    R::Task: Send,
{
    Then { endpoint, f }
}

#[derive(Copy, Clone, Debug)]
pub struct Then<E, F> {
    endpoint: E,
    f: F,
}

impl<E, F, R> Endpoint for Then<E, F>
where
    E: Endpoint,
    F: FnOnce(E::Output) -> R + Clone + Send,
    R: IntoTask,
    R::Task: Send,
{
    type Output = R::Output;
    type Task = ThenTask<E::Task, F, R>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Task> {
        let task = self.endpoint.apply(cx)?;
        Some(ThenTask::First(task, self.f.clone()))
    }
}

#[derive(Debug)]
pub enum ThenTask<T, F, R>
where
    T: Task,
    F: FnOnce(T::Output) -> R + Send,
    R: IntoTask,
    R::Task: Send,
{
    First(T, F),
    Second(R::Task),
    Done,
}

impl<T, F, R> Task for ThenTask<T, F, R>
where
    T: Task,
    F: FnOnce(T::Output) -> R + Send,
    R: IntoTask,
    R::Task: Send,
{
    type Output = R::Output;

    fn poll_task(&mut self, cx: &mut task::Context) -> PollTask<Self::Output> {
        use self::ThenTask::*;
        loop {
            // TODO: optimize
            match mem::replace(self, Done) {
                First(mut task, f) => match task.poll_task(cx) {
                    PollTask::Pending => {
                        *self = First(task, f);
                        return PollTask::Pending;
                    }
                    PollTask::Ready(r) => {
                        cx.input().enter_scope(|| {
                            *self = Second(f(r).into_task());
                        });
                        continue;
                    }
                    PollTask::Aborted(e) => return PollTask::Aborted(e),
                },
                Second(mut fut) => {
                    return match fut.poll_task(cx) {
                        PollTask::Pending => {
                            *self = Second(fut);
                            PollTask::Pending
                        }
                        polled => polled,
                    }
                }
                Done => panic!(),
            }
        }
    }
}
