#![allow(missing_docs)]
#![allow(non_snake_case)]

use super::{Context, Endpoint, IntoEndpoint};
use futures::{future, IntoFuture};
use request::Input;
use std::fmt;

macro_rules! generate {
    ($(
        ($new:ident, $Join:ident, $JoinResult:ident, <$($T:ident),*>),
    )*) => {$(
        pub fn $new<$($T),*>($($T: $T),*) -> $Join<$($T::Endpoint),*>
        where $(
            $T: IntoEndpoint,
        )*
        {
            $Join {
                $($T: $T.into_endpoint(),)*
            }
        }

        pub struct $Join<$($T),*> {
            $(
                $T: $T,
            )*
        }

        impl<$($T),*> Copy for $Join<$($T),*>
        where $(
            $T: Copy,
        )* {}

        impl<$($T),*> Clone for $Join<$($T),*>
        where $(
            $T: Clone,
        )* {
            fn clone(&self) -> Self {
                $Join {
                    $(
                        $T: self.$T.clone(),
                    )*
                }
            }
        }

        impl<$($T),*> fmt::Debug for $Join<$($T),*>
        where $(
            $T: fmt::Debug,
        )* {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.debug_tuple(stringify!($Join))
                $( .field(&self.$T) )*
                .finish()
            }
        }

        impl<$($T),*> Endpoint for $Join<$($T),*>
        where $(
            $T: Endpoint,
        )*
        {
            type Item = ($($T::Item),*);
            type Future = future::$Join<$($T::Future),*>;

            fn apply(&self, input: &Input, ctx: &mut Context) -> Option<Self::Future> {
                $(
                    let $T = self.$T.apply(input, ctx)?;
                )*
                Some(IntoFuture::into_future(($($T),*)))
            }
        }

        impl<$($T: IntoEndpoint),*> IntoEndpoint for ($($T),*) {
            type Item = ($($T::Item),*);
            type Endpoint = $Join<$($T::Endpoint),*>;

            fn into_endpoint(self) -> Self::Endpoint {
                let ($($T),*) = self;
                $new ($($T),*)
            }
        }
    )*};
}

generate! {
    (join,  Join, JoinResult, <E1, E2>),
    (join3, Join3, Join3Result, <E1, E2, E3>),
    (join4, Join4, Join4Result, <E1, E2, E3, E4>),
    (join5, Join5, Join5Result, <E1, E2, E3, E4, E5>),
}
