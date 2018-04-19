use futures::Async;

use endpoint::{Context, Endpoint};
use error::Error;
use input::{Input, RequestBody};
use task::{self, Task};

/// Create an asynchronous computation for handling an HTTP request.
pub fn apply<E>(endpoint: &E, input: &Input, body: RequestBody) -> Apply<E::Task>
where
    E: Endpoint,
{
    let in_flight = endpoint.apply(&mut Context::new(input));
    Apply {
        in_flight,
        body: Some(body),
    }
}

/// The type of value which will be returned from "apply".
///
/// Typically, this value is wrapped by a type which implements "Future"
/// and holds the instance of "Input".
#[derive(Debug)]
pub struct Apply<T> {
    in_flight: Option<T>,
    body: Option<RequestBody>,
}

impl<T: Task> Apply<T> {
    /// Poll the inner "Task" and return its output if available.
    pub fn poll_ready(&mut self, input: &Input) -> Async<Option<Result<T::Output, Error>>> {
        let result = match self.in_flight {
            Some(ref mut f) => match f.poll_task(&mut task::Context::new(input, &mut self.body)) {
                Ok(Async::NotReady) => return Async::NotReady,
                Ok(Async::Ready(ok)) => Some(Ok(ok)),
                Err(err) => Some(Err(err)),
            },
            None => None,
        };
        Async::Ready(result)
    }
}
