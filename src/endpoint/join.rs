#![allow(missing_docs)]
#![allow(non_snake_case)]

use std::fmt;
use futures::{future, IntoFuture};
use http::Request;
use super::{Endpoint, EndpointContext, EndpointResult, IntoEndpoint};

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
            type Result = $JoinResult<$($T::Result),*>;

            fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Result> {
                $(
                    let $T = try_opt!(self.$T.apply(ctx));
                )*
                Some($JoinResult { inner: ($($T),*) })
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

        #[derive(Debug)]
        pub struct $JoinResult<$($T),*> {
            inner: ($($T),*),
        }

        impl<$($T: EndpointResult),*> EndpointResult for $JoinResult<$($T),*> {
            type Item = ($($T::Item),*);
            type Future = future::$Join<$($T::Future),*>;

            fn into_future(self, request: &mut Request) -> Self::Future {
                let ($($T),*) = self.inner;
                $(
                    let $T = $T.into_future(request);
                )*
                IntoFuture::into_future(($($T),*))
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
