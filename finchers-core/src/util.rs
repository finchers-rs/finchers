use endpoint::{task, Context, Endpoint, Error};
use error::NoRoute;
use futures::{Async, Future, Poll};
use input::Input;

/// Create a task for processing an incoming HTTP request by using given `Endpoint`.
pub fn create_task<E: Endpoint>(endpoint: &E, input: Input) -> EndpointTask<E::Task> {
    let in_flight = endpoint.apply(&mut Context::new(&input));
    EndpointTask {
        input: Some(input),
        in_flight,
    }
}

#[derive(Debug)]
pub struct EndpointTask<F> {
    input: Option<Input>,
    in_flight: Option<F>,
}

impl<F: task::Task> Future for EndpointTask<F> {
    type Item = (Result<F::Output, Error>, Input);
    type Error = !;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let result = match self.in_flight {
            Some(ref mut f) => {
                let input = self.input.as_mut().expect("cannot resolve/reject twice");
                let mut cx = task::Context::new(input);
                match f.poll_task(&mut cx) {
                    Ok(Async::NotReady) => return Ok(Async::NotReady),
                    Ok(Async::Ready(ok)) => Ok(ok),
                    Err(err) => Err(err),
                }
            }
            None => Err(NoRoute::new().into()),
        };
        let input = self.input.take().expect("The instance of Input has gone.");
        Ok(Async::Ready((result, input)))
    }
}
