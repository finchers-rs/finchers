use finchers_core::error::NoRoute;
use finchers_core::input::replace_input;
use finchers_core::output::{Output, Responder};
use finchers_core::{Error, Input};
use futures::{Async, Future, Poll};
use {Context, Endpoint};

pub fn apply<E: Endpoint>(endpoint: &E, input: Input) -> Apply<E::Future> {
    let in_flight = endpoint.apply(&input, &mut Context::new(&input));
    Apply {
        input: Some(input),
        in_flight,
    }
}

#[derive(Debug)]
pub struct Apply<F> {
    input: Option<Input>,
    in_flight: Option<F>,
}

impl<F: Future<Error = Error>> Future for Apply<F> {
    type Item = F::Item;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if let Some(input) = self.input.take() {
            replace_input(Some(input));
        }
        match self.in_flight {
            Some(ref mut f) => f.poll(),
            None => Err(NoRoute::new().into()),
        }
    }
}

pub fn apply_and_respond<E>(endpoint: &E, input: Input) -> ApplyAndRespond<E::Future>
where
    E: Endpoint,
    E::Item: Responder,
{
    let in_flight = endpoint.apply(&input, &mut Context::new(&input));
    ApplyAndRespond {
        input: Some(input),
        in_flight,
    }
}

#[derive(Debug)]
pub struct ApplyAndRespond<F> {
    input: Option<Input>,
    in_flight: Option<F>,
}

impl<F> Future for ApplyAndRespond<F>
where
    F: Future<Error = Error>,
    F::Item: Responder,
{
    type Item = Output;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if let Some(input) = self.input.take() {
            replace_input(Some(input));
        }
        match self.in_flight {
            Some(ref mut f) => {
                let item = try_ready!(f.poll());
                Input::with(|input| item.respond(input).map(Async::Ready).map_err(Into::into))
            }
            None => Err(NoRoute::new().into()),
        }
    }
}
