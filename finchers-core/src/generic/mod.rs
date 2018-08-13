#![allow(missing_docs)]

mod combine;
mod either;
mod func;
mod hlist;

pub use self::combine::Combine;
pub use self::either::Either;
pub use self::func::Func;
pub use self::hlist::Tuple;

use std::fmt;
use std::marker::PhantomData;

pub type One<T> = (T,);

#[inline]
pub fn one<T>(x: T) -> One<T> {
    (x,)
}

#[inline]
pub fn map_one<T, U>(x: One<T>, f: impl FnOnce(T) -> U) -> One<U> {
    one(f(x.0))
}

pub struct MapLeft<R>(PhantomData<fn() -> R>);

impl<R> Copy for MapLeft<R> {}

impl<R> Clone for MapLeft<R> {
    fn clone(&self) -> Self {
        *self
    }
}

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

pub struct MapRight<L>(PhantomData<fn() -> L>);

impl<L> Copy for MapRight<L> {}

impl<L> Clone for MapRight<L> {
    fn clone(&self) -> Self {
        *self
    }
}
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
