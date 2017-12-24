#![allow(missing_docs)]
#![allow(non_snake_case)]

use futures::{future, IntoFuture};
use super::{Task, TaskContext};

macro_rules! generate {
    ($(
        ($new:ident, $Join:ident, <$($T:ident),*>),
    )*) => {
        $(
            #[derive(Debug)]
            pub struct $Join<$($T),*> {
              pub(crate)   inner: ($($T),*),
            }

            impl<$($T,)* E> Task for $Join<$($T),*>
            where $(
                $T: Task<Error = E>,
            )*
            {
                type Item = ($($T::Item),*);
                type Error = E;
                type Future = future::$Join<$($T::Future),*>;

                fn launch(self, ctx: &mut TaskContext) -> Self::Future {
                    let ($($T),*) = self.inner;
                    $(
                        let $T = $T.launch(ctx);
                    )*
                    ($($T),*).into_future()
                }
            }
        )*
    };
}

generate! {
    (join, Join, <T1, T2>),
    (join3, Join3, <T1, T2, T3>),
    (join4, Join4, <T1, T2, T3, T4>),
    (join5, Join5, <T1, T2, T3, T4, T5>),
}
