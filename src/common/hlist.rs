pub trait Tuple: Sized {
    type HList: HList<Tuple = Self>;

    fn hlist(self) -> Self::HList;
}

impl Tuple for () {
    type HList = HNil;

    #[inline(always)]
    fn hlist(self) -> Self::HList {
        HNil
    }
}

pub trait HList: Sized {
    type Tuple: Tuple<HList = Self>;

    fn tuple(self) -> Self::Tuple;
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct HNil;

#[allow(clippy::unused_unit)]
impl HList for HNil {
    type Tuple = ();

    #[inline(always)]
    fn tuple(self) -> Self::Tuple {
        ()
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct HCons<H, T: HList> {
    pub head: H,
    pub tail: T,
}

macro_rules! hcons {
    ($H:expr) => {
        HCons {
            head: $H,
            tail: HNil,
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
    ($H:ty) => { HCons<$H, HNil> };
    ($H:ty, $($T:ty),*) => { HCons<$H, HCons!($($T),*)> };
}

macro_rules! hcons_pat {
    ($H:pat) => {
        HCons {
            head: $H,
            tail: HNil,
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
    // T31, T30, T29, T28, T27, T26, T25, T24, T23, T22, T21, T20, T19, T18, T17, T16,
    T15, T14, T13, T12, T11, T10, T9, T8, T7, T6, T5, T4, T3, T2, T1, T0,
}
