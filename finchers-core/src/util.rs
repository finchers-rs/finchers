use endpoint::{Context, Endpoint, Error};
use error::NoRoute;
use futures::{Async, Future, Poll};
use input::{replace_input, Input};

/// Create a task for processing an incoming HTTP request by using given `Endpoint`.
pub fn create_task<E: Endpoint>(endpoint: &E, input: Input) -> EndpointTask<E::Future> {
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

impl<F: Future<Error = Error>> Future for EndpointTask<F> {
    type Item = (Result<F::Item, Error>, Input);
    type Error = !;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if let Some(input) = self.input.take() {
            replace_input(Some(input));
        }

        let result = match self.in_flight {
            Some(ref mut f) => match f.poll() {
                Ok(Async::NotReady) => return Ok(Async::NotReady),
                Ok(Async::Ready(ok)) => Ok(ok),
                Err(err) => Err(err),
            },
            None => Err(NoRoute::new().into()),
        };
        let input = replace_input(None).expect("The instance of Input has gone.");

        Ok(Async::Ready((result, input)))
    }
}
