use futures::Async;

use super::{Context, Endpoint};
use error::Error;
use input::{Input, RequestBody};
use task::{self, PollTask, Task};

/// Create an asynchronous computation for handling an HTTP request.
pub fn apply_request<E>(endpoint: &E, input: &Input, body: RequestBody) -> ApplyRequest<E::Task>
where
    E: Endpoint + ?Sized,
{
    let in_flight = endpoint.apply(&mut Context::new(input));
    ApplyRequest {
        in_flight,
        body: Some(body),
    }
}

/// An asynchronous computation created by the endpoint.
///
/// Typically, this value is wrapped by a type which contains an instance of `Input`.
#[derive(Debug)]
pub struct ApplyRequest<T> {
    in_flight: Option<T>,
    body: Option<RequestBody>,
}

impl<T: Task> ApplyRequest<T> {
    /// Poll the inner "Task" and return its output if available.
    pub fn poll_ready(&mut self, input: &Input) -> PollReady<Option<Result<T::Output, Error>>> {
        match self.in_flight {
            Some(ref mut f) => {
                let mut cx = task::Context::new(input, &mut self.body);
                match f.poll_task(&mut cx) {
                    PollTask::Pending => PollReady::Pending,
                    PollTask::Ready(ok) => PollReady::Ready(Some(Ok(ok))),
                    PollTask::Aborted(err) => PollReady::Ready(Some(Err(err))),
                }
            }
            None => PollReady::Ready(None),
        }
    }
}

// FIXME: replace with core::task::Poll
#[derive(Debug, Copy, Clone)]
pub enum PollReady<T> {
    Pending,
    Ready(T),
}

impl<T> Into<Async<T>> for PollReady<T> {
    fn into(self) -> Async<T> {
        match self {
            PollReady::Pending => Async::NotReady,
            PollReady::Ready(v) => Async::Ready(v),
        }
    }
}
