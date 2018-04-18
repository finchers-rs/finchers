use futures::{Async, Future, Poll};

use endpoint::{Context, Endpoint};
use error::Error;
use input::{Input, RequestBody};
use task::{self, Task};

/// Create a task for processing an incoming HTTP request by using given `Endpoint`.
pub fn create_task<E: Endpoint>(endpoint: &E, input: Input, body: RequestBody) -> EndpointTask<E::Task> {
    let in_flight = endpoint.apply(&mut Context::new(&input));
    EndpointTask {
        input: Some(input),
        body: Some(body),
        in_flight,
    }
}

#[derive(Debug)]
pub struct EndpointTask<F> {
    input: Option<Input>,
    body: Option<RequestBody>,
    in_flight: Option<F>,
}

impl<F: Task> Future for EndpointTask<F> {
    type Item = (Result<F::Output, Error>, Input);
    type Error = !;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let result = match self.in_flight {
            Some(ref mut f) => {
                let input = self.input.as_ref().expect("cannot resolve/reject twice");
                match f.poll_task(&mut task::Context::new(input, &mut self.body)) {
                    Ok(Async::NotReady) => return Ok(Async::NotReady),
                    Ok(Async::Ready(ok)) => Ok(ok),
                    Err(err) => Err(err),
                }
            }
            None => Err(Error::canceled()),
        };
        let input = self.input.take().expect("The instance of Input has gone.");
        Ok(Async::Ready((result, input)))
    }
}
