#![allow(missing_docs)]
#![allow(non_snake_case)]

use std::marker::PhantomData;
use endpoint::{Endpoint, EndpointContext, EndpointError, IntoEndpoint};
use task;


macro_rules! generate {
    ($(
        ($new:ident, $Join:ident, <$($T:ident : $A:ident),*>),
    )*) => {$(
        pub fn $new<$($T,)* $($A,)* E>($( $T: $T ),*) -> $Join <$($T::Endpoint,)* E>
        where $(
            $T: IntoEndpoint<$A, E>,
        )*
        {
            $Join {
                $($T: $T.into_endpoint(),)*
                _marker: PhantomData,
            }
        }

        #[derive(Debug)]
        pub struct $Join<$($T,)* E>
        where $(
            $T: Endpoint<Error = E>,
        )* {
            $(
                $T: $T,
            )*
            _marker: PhantomData<fn() -> E>,
        }

        impl<$($T,)* E> Endpoint for $Join<$($T,)* E>
        where $(
            $T: Endpoint<Error = E>,
        )*
        {
            type Item = ($($T::Item),*);
            type Error = E;
            type Task = task::$Join<$($T::Task,)* E>;

            fn apply(&self, ctx: &mut EndpointContext) -> Result<Self::Task, EndpointError> {
                $(
                    let $T = self.$T.apply(ctx)?;
                )*
                Ok(task::$new($($T),*))
            }
        }

        impl<$($T,)* $($A,)* E> IntoEndpoint<($($A),*), E> for ($($T),*)
        where $(
            $T: IntoEndpoint<$A, E>,
        )* {
            type Endpoint = $Join<$($T::Endpoint,)* E>;

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
    (join6, Join6, <E1:T1, E2:T2, E3:T3, E4:T4, E5:T5, E6:T6>),
}
