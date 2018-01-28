#![allow(missing_docs)]
#![allow(non_snake_case)]

use std::fmt;
use futures::{future, IntoFuture};
use http::Request;
use super::{Endpoint, EndpointContext, EndpointResult, IntoEndpoint};
use errors::HttpError;

macro_rules! generate {
    ($(
        ($new:ident, $Join:ident, $JoinResult:ident, <$($T:ident : $A:ident),*>),
    )*) => {$(
        pub fn $new<$($T,)* $($A,)* E: HttpError>($($T: $T),*) -> $Join<$($T::Endpoint),*>
        where $(
            $T: IntoEndpoint<$A, E>,
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

        impl<$($T,)* E: HttpError> Endpoint for $Join<$($T),*>
        where $(
            $T: Endpoint<Error = E>,
        )*
        {
            type Item = ($($T::Item),*);
            type Error = E;
            type Result = $JoinResult<$($T::Result),*>;

            fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Result> {
                $(
                    let $T = try_opt!(self.$T.apply(ctx));
                )*
                Some($JoinResult { inner: ($($T),*) })
            }
        }

        impl<$($T,)* $($A,)* E: HttpError> IntoEndpoint<($($A),*), E> for ($($T),*)
        where $(
            $T: IntoEndpoint<$A, E>,
        )* {
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

        impl<$($T,)* E: HttpError> EndpointResult for $JoinResult<$($T),*>
        where $(
            $T: EndpointResult<Error = E>,
        )*
        {
            type Item = ($($T::Item),*);
            type Error = E;
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
    (join,  Join, JoinResult, <E1:T1, E2:T2>),
    (join3, Join3, Join3Result, <E1:T1, E2:T2, E3:T3>),
    (join4, Join4, Join4Result, <E1:T1, E2:T2, E3:T3, E4:T4>),
    (join5, Join5, Join5Result, <E1:T1, E2:T2, E3:T3, E4:T4, E5:T5>),
}
