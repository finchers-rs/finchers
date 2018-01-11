#![allow(missing_docs)]

use endpoint::{Endpoint, EndpointContext, IntoEndpoint};
use task;

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

#[derive(Debug)]
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
    type Task = task::or::Or<E1::Task, E2::Task>;

    fn apply(&self, ctx2: &mut EndpointContext) -> Option<Self::Task> {
        let mut ctx1 = ctx2.clone();
        let t1 = self.e1.apply(&mut ctx1);
        let t2 = self.e2.apply(ctx2);
        match (t1, t2) {
            (Some(t1), Some(t2)) => {
                // If both endpoints are matched, the one with the larger number of
                // (consumed) path segments is choosen.
                let inner = if ctx1.segments().popped() > ctx2.segments().popped() {
                    *ctx2 = ctx1;
                    task::or::Left(t1)
                } else {
                    task::or::Right(t2)
                };
                Some(task::or::Or { inner })
            }
            (Some(t1), None) => {
                *ctx2 = ctx1;
                Some(task::or::Or {
                    inner: task::or::Left(t1),
                })
            }
            (None, Some(t2)) => Some(task::or::Or {
                inner: task::or::Right(t2),
            }),
            (None, None) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hyper::{Method, Request};
    use endpoint::result::ok;
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
