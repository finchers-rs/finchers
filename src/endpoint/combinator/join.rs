#![allow(missing_docs)]
#![allow(non_snake_case)]

use std::marker::PhantomData;
use context::Context;
use endpoint::{Endpoint, EndpointError};
use task;


macro_rules! generate {
    ($(
        ($new:ident, $Join:ident, <$($T:ident),*>),
    )*) => {$(
        pub fn $new<$($T,)* E>($( $T: $T ),*) -> $Join <$($T,)* E>
        where $(
            $T: Endpoint<Error = E>,
        )*
        {
            $Join {
                $($T,)*
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
            _marker: PhantomData<E>,
        }

        impl<$($T,)* E> Endpoint for $Join<$($T,)* E>
        where $(
            $T: Endpoint<Error = E>,
        )*
        {
            type Item = ($($T::Item),*);
            type Error = E;
            type Task = task::$Join<$($T::Task,)* E>;

            fn apply(&self, ctx: &mut Context) -> Result<Self::Task, EndpointError> {
                $(
                    let $T = self.$T.apply(ctx)?;
                )*
                Ok(task::$new($($T),*))
            }
        }
    )*};
}

generate! {
    (join, Join, <E1, E2>),
    (join3, Join3, <E1, E2, E3>),
    (join4, Join4, <E1, E2, E3, E4>),
    (join5, Join5, <E1, E2, E3, E4, E5>),
    (join6, Join6, <E1, E2, E3, E4, E5, E6>),
}
