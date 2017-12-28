#![allow(missing_docs)]
#![allow(non_snake_case)]

use endpoint::{Endpoint, EndpointContext, IntoEndpoint};
use task;

macro_rules! generate {
    ($(
        ($new:ident, $Join:ident, <$($T:ident : $A:ident),*>),
    )*) => {$(
        pub fn $new<$($T,)* $($A,)* E>($($T: $T),*) -> $Join<$($T::Endpoint),*>
        where $(
            $T: IntoEndpoint<$A, E>,
        )*
        {
            $Join {
                $($T: $T.into_endpoint(),)*
            }
        }

        #[derive(Debug)]
        pub struct $Join<$($T),*> {
            $(
                $T: $T,
            )*
        }

        impl<$($T,)* E> Endpoint for $Join<$($T),*>
        where $(
            $T: Endpoint<Error = E>,
        )*
        {
            type Item = ($($T::Item),*);
            type Error = E;
            type Task = task::join::$Join<$($T::Task),*>;

            fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Task> {
                $(
                    let $T = try_opt!(self.$T.apply(ctx));
                )*
                Some(task::join::$Join { inner: ($($T),*) })
            }
        }

        impl<$($T,)* $($A,)* E> IntoEndpoint<($($A),*), E> for ($($T),*)
        where $(
            $T: IntoEndpoint<$A, E>,
        )* {
            type Endpoint = $Join<$($T::Endpoint),*>;

            fn into_endpoint(self) -> Self::Endpoint {
                let ($($T),*) = self;
                $new ($($T),*)
            }
        }
    )*};
}

generate! {
    (join,  Join,  <E1:T1, E2:T2>),
    (join3, Join3, <E1:T1, E2:T2, E3:T3>),
    (join4, Join4, <E1:T1, E2:T2, E3:T3, E4:T4>),
    (join5, Join5, <E1:T1, E2:T2, E3:T3, E4:T4, E5:T5>),
}
