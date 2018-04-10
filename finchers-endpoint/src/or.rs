use finchers_core::Input;
use finchers_core::endpoint::{Context, Endpoint, IntoEndpoint};
use futures::{Future, Poll};

pub fn new<E1, E2>(e1: E1, e2: E2) -> Or<E1::Endpoint, E2::Endpoint>
where
    E1: IntoEndpoint,
    E2: IntoEndpoint<Item = E1::Item>,
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
    E2: Endpoint<Item = E1::Item>,
{
    type Item = E1::Item;
    type Future = OrFuture<E1::Future, E2::Future>;

    fn apply(&self, input: &Input, ctx2: &mut Context) -> Option<Self::Future> {
        let mut ctx1 = ctx2.clone();
        let t1 = self.e1.apply(input, &mut ctx1);
        let t2 = self.e2.apply(input, ctx2);
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
                Some(OrFuture { inner })
            }
            (Some(t1), None) => {
                *ctx2 = ctx1;
                Some(OrFuture {
                    inner: Either::Left(t1),
                })
            }
            (None, Some(t2)) => Some(OrFuture {
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
