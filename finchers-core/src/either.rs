#![allow(missing_docs)]

#[derive(Debug, Copy, Clone)]
pub enum Either<L, R> {
    Left(L),
    Right(R),
}
