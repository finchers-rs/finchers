#![allow(missing_docs)]

//! Definition of `Either`s

use std::fmt::{self, Display};
use std::error::Error;
use futures::{Async, Future, Poll};
use context::Context;
use response::{Responder, Response};

macro_rules! define_either {
    ($name:ident <$( $variant:ident ),*>) => {
        #[derive(Debug)]
        pub enum $name<$( $variant ),*> {
            $(
                $variant($variant),
            )*
        }

        impl<$( $variant ),*> Display for $name<$( $variant ),*>
        where
        $( $variant: Display ),*
        {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                match *self {
                    $(
                        $name :: $variant (ref e) =>
                            write!(f,
                                   concat!(stringify!($name), "::", stringify!($variant),
                                   "({})"),
                                   e),
                    )*
                }
            }
        }

        impl<$( $variant ),*> Error for $name<$( $variant ),*>
        where
        $( $variant: Error ),*
        {
            fn description(&self) -> &str {
                match *self {
                    $(
                        $name :: $variant (ref e) => e.description(),
                    )*
                }
            }
        }

        impl<$( $variant ),*> Future for $name<$( $variant ),*>
        where
        $( $variant: Future ),*
        {
            type Item = $name <$( $variant :: Item ),*>;
            type Error = $name <$( $variant :: Error ),*>;

            fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
                match *self {
                    $(
                        $name :: $variant(ref mut e) => {
                            let item = try_ready!(e.poll().map_err($name :: $variant));
                            Ok(Async::Ready($name :: $variant (item)))
                        },
                    )*
                }
            }
        }

        impl<$( $variant ),*> Responder for $name<$( $variant ),*>
        where
        $( $variant: Responder ),*
        {
            fn respond_to(&mut self, ctx: &mut Context) -> Response {
                match *self {
                    $(
                        $name :: $variant (ref mut e) => e.respond_to(ctx),
                    )*
                }
            }
        }
    }
}

define_either!(Either2<E1, E2>);
define_either!(Either3<E1, E2, E3>);
define_either!(Either4<E1, E2, E3, E4>);
define_either!(Either5<E1, E2, E3, E4, E5>);
define_either!(Either6<E1, E2, E3, E4, E5, E6>);
define_either!(Either7<E1, E2, E3, E4, E5, E6, E7>);
