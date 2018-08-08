#![allow(missing_docs)]

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Either<L, R> {
    Left(L),
    Right(R),
}
