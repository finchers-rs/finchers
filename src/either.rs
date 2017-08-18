use futures::{Future, Poll};


#[derive(Debug)]
pub enum Either<A, B> {
    A(A),
    B(B),
}

impl<A, B> Future for Either<A, B>
where
    A: Future,
    B: Future<Error = A::Error>,
{
    type Item = Either<A::Item, B::Item>;
    type Error = A::Error;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match *self {
            Either::A(ref mut a) => Ok(Either::A(try_ready!(a.poll())).into()),
            Either::B(ref mut b) => Ok(Either::B(try_ready!(b.poll())).into()),
        }
    }
}
