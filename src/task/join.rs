#![allow(missing_docs)]
#![allow(non_snake_case)]

use std::fmt;
use std::marker::PhantomData;

use super::{Async, Poll, Task, TaskContext};
use super::maybe_done::MaybeDone;

// TODO: add Join3, Join4, Join5

macro_rules! generate {
    ($(
        ($new:ident, $Join:ident, <$($T:ident),*>),
    )*) => {
        $(
            pub fn $new <$( $T, )* E>($($T: $T),*)
                -> $Join <$( $T, )* E>
            where $(
                $T: Task<Error = E>,
            )*
            {
                $Join {
                    $(
                        $T: MaybeDone::NotYet($T),
                    )*
                    _marker: PhantomData,
                }
            }

            pub struct $Join<$($T),*, E>
            where $(
                $T: Task<Error = E>,
            )*
            {
                $( $T: MaybeDone<$T>, )*
                _marker: PhantomData<E>,
            }

            impl<$($T,)* E> fmt::Debug for $Join<$($T,)* E>
            where $(
                $T: Task<Error = E> + fmt::Debug,
                $T::Item: fmt::Debug,
            )*
            {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    f.debug_struct(stringify!(Join<T1, T2>))
                        $(
                            .field(stringify!($T), &self.$T)
                        )*
                        .finish()
                }
            }

            impl<$($T,)* E> $Join<$($T,)* E>
            where $(
                $T: Task<Error = E>,
            )*
            {
                fn erase(&mut self) {
                    $(
                        self.$T = MaybeDone::Gone;
                    )*
                }
            }

            impl<$($T,)* E> Task for $Join<$($T,)* E>
            where $(
                $T: Task<Error = E>,
            )*
            {
                type Item = ($($T::Item),*);
                type Error = E;

                fn poll(&mut self, ctx: &mut TaskContext) -> Poll<Self::Item, Self::Error> {
                    let mut all_done = true;
                    $(
                        all_done = all_done && match self.$T.poll(ctx) {
                            Ok(done) => done,
                            Err(e) => {
                                self.erase();
                                return Err(e);
                            }
                        };
                    )*

                    if all_done {
                        Ok(Async::Ready(($(self.$T.take()),*)))
                    } else {
                        Ok(Async::NotReady)
                    }
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
    (join6, Join6, <T1, T2, T3, T4, T5, T6>),
}
