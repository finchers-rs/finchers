#![allow(missing_docs)]

use bytes::Buf;
use futures::Poll;
use hyper::body::Payload;
use std::error::Error as StdError;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Either<L, R> {
    Left(L),
    Right(R),
}

impl<L: Buf, R: Buf> Buf for Either<L, R> {
    fn remaining(&self) -> usize {
        match self {
            Either::Left(ref t) => t.remaining(),
            Either::Right(ref t) => t.remaining(),
        }
    }

    fn bytes(&self) -> &[u8] {
        match self {
            Either::Left(ref t) => t.bytes(),
            Either::Right(ref t) => t.bytes(),
        }
    }

    fn advance(&mut self, cnt: usize) {
        match self {
            Either::Left(ref mut t) => t.advance(cnt),
            Either::Right(ref mut t) => t.advance(cnt),
        }
    }
}

impl<L: Payload, R: Payload> Payload for Either<L, R> {
    type Data = Either<L::Data, R::Data>;
    type Error = Box<StdError + Send + Sync>;

    fn poll_data(&mut self) -> Poll<Option<Self::Data>, Self::Error> {
        match self {
            Either::Left(ref mut t) => t
                .poll_data()
                .map(|x| x.map(|data| data.map(Either::Left)))
                .map_err(Into::into),
            Either::Right(ref mut t) => t
                .poll_data()
                .map(|x| x.map(|data| data.map(Either::Right)))
                .map_err(Into::into),
        }
    }
}
