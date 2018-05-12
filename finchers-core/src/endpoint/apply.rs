use super::{Context, Endpoint};
use error::Error;
use input::{Input, RequestBody};
use poll::Poll;
use task::{self, Task};

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
    /// Poll the inner `Task` and return its output if available.
    pub fn poll_ready(&mut self, input: &Input) -> Poll<Result<T::Output, Error>> {
        match self.in_flight {
            Some(ref mut f) => {
                let mut cx = task::Context::new(input, &mut self.body);
                f.poll_task(&mut cx)
            }
            None => Poll::Ready(Err(Error::skipped())),
        }
    }
}
