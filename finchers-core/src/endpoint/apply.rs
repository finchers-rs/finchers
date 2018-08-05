use super::{Context, Endpoint};
use crate::error::Error;
use crate::input::{with_set_cx, Input};
use crate::poll::Poll;
use crate::task::Task;

/// Create an asynchronous computation for handling an HTTP request.
pub fn apply_request<E>(endpoint: &E, input: &Input) -> ApplyRequest<E::Task>
where
    E: Endpoint + ?Sized,
{
    let in_flight = endpoint.apply(&mut Context::new(input));
    ApplyRequest { in_flight }
}

/// An asynchronous computation created by the endpoint.
///
/// Typically, this value is wrapped by a type which contains an instance of `Input`.
#[derive(Debug)]
pub struct ApplyRequest<T> {
    in_flight: Option<T>,
}

impl<T: Task> ApplyRequest<T> {
    /// Poll the inner `Task` and return its output if available.
    pub fn poll_ready(&mut self, input: &mut Input) -> Poll<Result<T::Output, Error>> {
        match self.in_flight {
            Some(ref mut f) => with_set_cx(input, || f.poll_task()),
            None => Poll::Ready(Err(Error::skipped())),
        }
    }
}
