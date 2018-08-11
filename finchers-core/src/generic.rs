#![allow(missing_docs)]

mod combine;
mod func;
mod hlist;

pub use self::combine::Combine;
pub use self::func::Func;
pub use self::hlist::Tuple;

pub type One<T> = (T,);

#[inline]
pub fn one<T>(x: T) -> One<T> {
    (x,)
}

#[inline]
pub fn map_one<T, U>(x: One<T>, f: impl FnOnce(T) -> U) -> One<U> {
    one(f(x.0))
}

use crate::either::Either;
use std::fmt;
use std::marker::PhantomData;

#[derive(Copy, Clone)]
pub struct MapLeft<R>(PhantomData<fn() -> R>);

impl<R> fmt::Debug for MapLeft<R> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("MapLeft").finish()
    }
}

impl<L: Tuple, R: Tuple> Func<L> for MapLeft<R> {
    type Out = (Either<L, R>,);

    #[inline(always)]
    fn call(self, args: L) -> Self::Out {
        (Either::Left(args),)
    }
}

#[derive(Copy, Clone)]
pub struct MapRight<L>(PhantomData<fn() -> L>);

impl<L> fmt::Debug for MapRight<L> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("MapRight").finish()
    }
}

impl<L: Tuple, R: Tuple> Func<R> for MapRight<L> {
    type Out = (Either<L, R>,);

    #[inline(always)]
    fn call(self, args: R) -> Self::Out {
        (Either::Right(args),)
    }
}

pub fn map_left<R>() -> MapLeft<R> {
    MapLeft(PhantomData)
}

pub fn map_right<L>() -> MapRight<L> {
    MapRight(PhantomData)
}
