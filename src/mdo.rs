#![allow(missing_docs)]

use futures::IntoFuture;

use errors::FinchersError;
use endpoint::Endpoint;
use endpoint::combinator::AndThen;
use endpoint::combinator::{ok, EndpointOk};


pub fn bind<E, F, Fut, R>(e: E, f: F) -> AndThen<E, F>
where
    E: Endpoint,
    F: FnOnce(E::Item) -> Fut,
    Fut: IntoFuture<Item = R, Error = FinchersError>,
{
    e.and_then(f)
}

pub fn ret<T>(x: T) -> EndpointOk<T> {
    ok(x)
}
