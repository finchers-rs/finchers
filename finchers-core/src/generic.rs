#![allow(missing_docs)]

// ==== HList ====

pub trait HList: Sized {
    type Tuple: Tuple<HList = Self>;

    fn tuple(self) -> Self::Tuple;
}

impl HList for () {
    type Tuple = ();

    #[inline(always)]
    fn tuple(self) -> Self::Tuple {
        ()
    }
}

pub trait Tuple: Sized {
    type HList: HList<Tuple = Self>;

    fn hlist(self) -> Self::HList;
}

impl Tuple for () {
    type HList = ();

    #[inline(always)]
    fn hlist(self) -> Self::HList {
        ()
    }
}

// == Combine ==

pub trait CombineBase<T: HList> {
    type Out: HList;

    fn combine(self, other: T) -> Self::Out;
}

// forall T:Hlist => (T, ()) -> T
impl<T: HList> CombineBase<T> for () {
    type Out = T;

    fn combine(self, other: T) -> Self::Out {
        other
    }
}

impl<H, T: HList, U: HList> CombineBase<U> for HCons<H, T>
where
    T: CombineBase<U>,
    HCons<H, <T as CombineBase<U>>::Out>: HList,
{
    type Out = HCons<H, <T as CombineBase<U>>::Out>;

    #[inline(always)]
    fn combine(self, other: U) -> Self::Out {
        HCons {
            head: self.head,
            tail: self.tail.combine(other),
        }
    }
}

pub trait Combine<T: Tuple>: Tuple + sealed::Sealed<T> {
    type Out: Tuple;

    fn combine(self, other: T) -> Self::Out;
}

impl<H: Tuple, T: Tuple> Combine<T> for H
where
    H::HList: CombineBase<T::HList>,
{
    type Out = <<H::HList as CombineBase<T::HList>>::Out as HList>::Tuple;

    fn combine(self, other: T) -> Self::Out {
        self.hlist().combine(other.hlist()).tuple()
    }
}

mod sealed {
    use super::{CombineBase, Tuple};

    pub trait Sealed<T> {}

    impl<H: Tuple, T: Tuple> Sealed<T> for H where H::HList: CombineBase<T::HList> {}
}

// == HCons ===

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct HCons<H, T: HList> {
    pub head: H,
    pub tail: T,
}

macro_rules! hcons {
    ($H:expr) => {
        HCons {
            head: $H,
            tail: (),
        }
    };
    ($H:expr, $($T:expr),*) => {
        HCons {
            head: $H,
            tail: hcons!($($T),*),
        }
    };
}

macro_rules! HCons {
    ($H:ty) => { HCons<$H, ()> };
    ($H:ty, $($T:ty),*) => { HCons<$H, HCons!($($T),*)> };
}

macro_rules! hcons_pat {
    ($H:pat) => {
        HCons {
            head: $H,
            tail: (),
        }
    };
    ($H:pat, $($T:pat),*) => {
        HCons {
            head: $H,
            tail: hcons_pat!($($T),*),
        }
    };
}

// == generics ==

macro_rules! generics {
    ($T:ident) => {
        impl<$T> HList for HCons!($T) {
            type Tuple = ($T,);

            #[inline(always)]
            fn tuple(self) -> Self::Tuple {
                (self.head,)
            }
        }

        impl<$T> Tuple for ($T,) {
            type HList = HCons!($T);

            #[inline(always)]
            fn hlist(self) -> Self::HList {
                hcons!(self.0)
            }
        }
    };
    ($H:ident, $($T:ident),*) => {
        generics!($($T),*);

        impl<$H, $($T),*> HList for HCons!($H, $($T),*) {
            type Tuple = ($H, $($T),*);

            #[inline(always)]
            fn tuple(self) -> Self::Tuple {
                #[allow(non_snake_case)]
                let hcons_pat!($H, $($T),*) = self;
                ($H, $($T),*)
            }
        }

        impl<$H, $($T),*> Tuple for ($H, $($T),*) {
            type HList = HCons!($H, $($T),*);

            #[inline(always)]
            fn hlist(self) -> Self::HList {
                #[allow(non_snake_case)]
                let ($H, $($T),*) = self;
                hcons!($H, $($T),*)
            }
        }
    };
    ($H:ident, $($T:ident,)*) => {
        generics!($H, $($T),*);
    };
}

generics! {
    T31, T30, T29, T28, T27, T26, T25, T24, T23, T22, T21, T20, T19, T18, T17, T16,
    T15, T14, T13, T12, T11, T10, T9, T8, T7, T6, T5, T4, T3, T2, T1, T0,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn combine<H: Tuple, T: Tuple>(h: H, t: T) -> H::Out
    where
        H: Combine<T>,
    {
        h.combine(t)
    }

    #[test]
    fn case1_units() {
        let a = ();
        let b = ();
        assert_eq!(combine(a, b), ());
    }

    #[test]
    fn case2_unit1() {
        let a = (10,);
        let b = ();
        assert_eq!(combine(a, b), (10,));
    }

    #[test]
    fn case3_unit2() {
        let a = ();
        let b = (10,);
        assert_eq!(combine(a, b), (10,));
    }

    #[test]
    fn case4_complicated() {
        let a = ("a", "b", "c");
        let b = (10, 20, 30);
        assert_eq!(combine(a, b), ("a", "b", "c", 10, 20, 30));
    }

    #[test]
    fn case5_nested() {
        let a = ("a", ("b", "c"));
        let b = (10, (20,), 30);
        assert_eq!(combine(a, b), ("a", ("b", "c"), 10, (20,), 30));
    }
}
