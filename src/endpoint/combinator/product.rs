#![allow(missing_docs)]

use futures::Future;
use futures::future::{Join, Join3, Join4, Join5};

use context::Context;
use endpoint::{Endpoint, EndpointResult};

macro_rules! define_product {
    ($fut:ident <$($type:ident),*>, ($($var:ident),*) => $($ret:tt)*) => {
        impl<$($type),*> Endpoint for ($($type),*)
        where
        $( $type: Endpoint, )*
        {
            type Item = ($( $type :: Item, )*);
            type Future = $fut <$( $type :: Future ),*>;

            fn apply(self, ctx: &mut Context) -> EndpointResult<Self::Future> {
                let ($($var),*) = self;
                $(
                    let $var = $var.apply(ctx)?;
                )*
                Ok( $($ret)* )
            }
        }
    }
}

define_product!(Join<A, B>, (a, b) => a.join(b));
define_product!(Join3<A, B, C>, (a, b, c) => a.join3(b, c));
define_product!(Join4<A, B, C, D>, (a, b, c, d) => a.join4(b, c, d));
define_product!(Join5<A, B, C, D, E>, (a, b, c, d, e) => a.join5(b, c, d, e));
