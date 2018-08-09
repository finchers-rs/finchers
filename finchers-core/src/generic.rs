#![allow(missing_docs)]

mod combine;
mod func;
mod hlist;

pub use self::combine::{Combine, CombineBase};
pub use self::func::Func;
pub use self::hlist::{HCons, HList, Tuple};

pub type One<T> = (T,);

#[inline]
pub fn one<T>(x: T) -> One<T> {
    (x,)
}

#[inline]
pub fn map_one<T, U>(x: One<T>, f: impl FnOnce(T) -> U) -> One<U> {
    one(f(x.0))
}
