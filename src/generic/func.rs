use super::hlist::Tuple;

pub trait Func<Args: Tuple> {
    type Out;

    fn call(&self, args: Args) -> Self::Out;
}

impl<F, R> Func<()> for F
where
    F: Fn() -> R,
{
    type Out = R;

    #[inline]
    fn call(&self, _: ()) -> Self::Out {
        (*self)()
    }
}

macro_rules! generics {
    ($T:ident) => {
        impl<F, R, $T> Func<($T,)> for F
        where
            F: Fn($T) -> R,
        {
            type Out = R;

            #[inline]
            fn call(&self, args: ($T,)) -> Self::Out {
                (*self)(args.0)
            }
        }
    };
    ($H:ident, $($T:ident),*) => {
        generics!($($T),*);

        impl<F, R, $H, $($T),*> Func<($H, $($T),*)> for F
        where
            F: Fn($H, $($T),*) -> R,
        {
            type Out = R;

            #[inline]
            fn call(&self, args: ($H, $($T),*)) -> Self::Out {
                #[allow(non_snake_case)]
                let ($H, $($T),*) = args;
                (*self)($H, $($T),*)
            }
        }
    };

    ($H:ident, $($T:ident,)*) => { generics! { $H, $($T),* } };
}

generics! {
    T15, T14, T13, T12, T11, T10, T9, T8, T7, T6, T5, T4, T3, T2, T1, T0,
}
