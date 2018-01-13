#![allow(missing_docs)]

use futures::{Future, Poll};
use http::Request;
use super::{Endpoint, EndpointContext, EndpointResult, IntoEndpoint};

pub fn or<E1, E2, A, B>(e1: E1, e2: E2) -> Or<E1::Endpoint, E2::Endpoint>
where
    E1: IntoEndpoint<A, B>,
    E2: IntoEndpoint<A, B>,
{
    Or {
        e1: e1.into_endpoint(),
        e2: e2.into_endpoint(),
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Or<E1, E2> {
    e1: E1,
    e2: E2,
}

impl<E1, E2> Endpoint for Or<E1, E2>
where
    E1: Endpoint,
    E2: Endpoint<Item = E1::Item, Error = E1::Error>,
{
    type Item = E1::Item;
    type Error = E1::Error;
    type Result = OrResult<E1::Result, E2::Result>;

    fn apply(&self, ctx2: &mut EndpointContext) -> Option<Self::Result> {
        let mut ctx1 = ctx2.clone();
        let t1 = self.e1.apply(&mut ctx1);
        let t2 = self.e2.apply(ctx2);
        match (t1, t2) {
            (Some(t1), Some(t2)) => {
                // If both endpoints are matched, the one with the larger number of
                // (consumed) path segments is choosen.
                let inner = if ctx1.segments().popped() > ctx2.segments().popped() {
                    *ctx2 = ctx1;
                    Either::Left(t1)
                } else {
                    Either::Right(t2)
                };
                Some(OrResult { inner })
            }
            (Some(t1), None) => {
                *ctx2 = ctx1;
                Some(OrResult {
                    inner: Either::Left(t1),
                })
            }
            (None, Some(t2)) => Some(OrResult {
                inner: Either::Right(t2),
            }),
            (None, None) => None,
        }
    }
}

#[derive(Debug)]
enum Either<T1, T2> {
    Left(T1),
    Right(T2),
}

#[derive(Debug)]
pub struct OrResult<T1, T2> {
    inner: Either<T1, T2>,
}

impl<T1, T2> EndpointResult for OrResult<T1, T2>
where
    T1: EndpointResult,
    T2: EndpointResult<Item = T1::Item, Error = T1::Error>,
{
    type Item = T1::Item;
    type Error = T1::Error;
    type Future = OrFuture<T1::Future, T2::Future>;

    fn into_future(self, request: &mut Request) -> Self::Future {
        match self.inner {
            Either::Left(t) => OrFuture {
                inner: Either::Left(t.into_future(request)),
            },
            Either::Right(t) => OrFuture {
                inner: Either::Right(t.into_future(request)),
            },
        }
    }
}

#[derive(Debug)]
pub struct OrFuture<T1, T2> {
    inner: Either<T1, T2>,
}

impl<T1, T2> Future for OrFuture<T1, T2>
where
    T1: Future,
    T2: Future<Item = T1::Item, Error = T1::Error>,
{
    type Item = T1::Item;
    type Error = T1::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match self.inner {
            Either::Left(ref mut e) => e.poll(),
            Either::Right(ref mut e) => e.poll(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hyper::{Method, Request};
    use endpoint::ok;
    use test::TestRunner;

    #[test]
    fn test_or_1() {
        let endpoint = e!("foo")
            .with(ok::<_, ()>("foo"))
            .or(e!("bar").with(ok("bar")));
        let mut runner = TestRunner::new(endpoint).unwrap();

        let request = Request::new(Method::Get, "/foo".parse().unwrap());
        match runner.run(request) {
            Some(Ok("foo")) => (),
            _ => panic!("does not match"),
        }

        let request = Request::new(Method::Get, "/bar".parse().unwrap());
        match runner.run(request) {
            Some(Ok("bar")) => (),
            _ => panic!("does not match"),
        }
    }

    #[test]
    fn test_or_choose_longer_segments() {
        let e1 = e!("foo").with(ok("foo"));
        let e2 = e!("foo/bar").with(ok::<_, ()>("foobar"));
        let endpoint = e1.or(e2);
        let mut runner = TestRunner::new(endpoint).unwrap();

        let request = Request::new(Method::Get, "/foo".parse().unwrap());
        match runner.run(request) {
            Some(Ok("foo")) => (),
            _ => panic!("does not match"),
        }

        let request = Request::new(Method::Get, "/foo/bar".parse().unwrap());
        match runner.run(request) {
            Some(Ok("foobar")) => (),
            _ => panic!("does not match"),
        }
    }
}
