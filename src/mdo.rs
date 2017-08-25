#![allow(missing_docs)]

use futures::IntoFuture;

use errors::FinchersError;
use endpoint::Endpoint;
use endpoint::endpoint::{value, Value};
use endpoint::combinator::AndThen;


pub fn bind<E, F, Fut, R>(e: E, f: F) -> AndThen<E, F>
where
    E: Endpoint,
    F: FnOnce(E::Item) -> Fut,
    Fut: IntoFuture<Item = R, Error = FinchersError>,
{
    e.and_then(f)
}

pub fn ret<T>(x: T) -> Value<T> {
    value(x)
}
