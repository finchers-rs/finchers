use endpoint::{Context, Endpoint, Error};
use error::NoRoute;
use futures::{Async, Future, Poll};
use input::Input;
use std::cell::RefCell;

/// Create a task for processing an incoming HTTP request by using given `Endpoint`.
pub fn create_task<E: Endpoint>(endpoint: &E, input: Input) -> EndpointTask<E::Future> {
    let in_flight = endpoint.apply(&mut Context::new(&input));
    EndpointTask {
        input: Some(RefCell::new(input)),
        in_flight,
    }
}

#[derive(Debug)]
pub struct EndpointTask<F> {
    input: Option<RefCell<Input>>,
    in_flight: Option<F>,
}

impl<F: Future<Error = Error>> Future for EndpointTask<F> {
    type Item = (Result<F::Item, Error>, Input);
    type Error = !;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let in_flight = &mut self.in_flight;
        let polled = Input::set(
            self.input.as_ref().expect("cannot resolve/reject twice"),
            || match *in_flight {
                Some(ref mut f) => f.poll(),
                None => Err(NoRoute::new().into()),
            },
        );
        let result = match polled {
            Ok(Async::NotReady) => return Ok(Async::NotReady),
            Ok(Async::Ready(ok)) => Ok(ok),
            Err(err) => Err(err),
        };
        let input = self.input.take().expect("The instance of Input has gone.");
        Ok(Async::Ready((result, input.into_inner())))
    }
}
