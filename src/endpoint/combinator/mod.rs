//! Definition of combinators

pub mod either;
pub mod ok;

use futures::{Future, IntoFuture};
use futures::future::{self, Join, Join3, Join4, Join5};

use context::Context;
use errors::FinchersError;
use endpoint::{Endpoint, EndpointResult};
use self::either::Either2;

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


#[allow(missing_docs)]
pub struct With<E1, E2>(pub(crate) E1, pub(crate) E2);

impl<E1, E2> Endpoint for With<E1, E2>
where
    E1: Endpoint,
    E2: Endpoint,
{
    type Item = E2::Item;
    type Future = E2::Future;

    fn apply(self, ctx: &mut Context) -> EndpointResult<Self::Future> {
        let _ = self.0.apply(ctx)?;
        let b = self.1.apply(ctx)?;
        Ok(b)
    }
}


#[allow(missing_docs)]
pub struct Skip<E1, E2>(pub(crate) E1, pub(crate) E2);

impl<E1, E2> Endpoint for Skip<E1, E2>
where
    E1: Endpoint,
    E2: Endpoint,
{
    type Item = E1::Item;
    type Future = E1::Future;

    fn apply(self, ctx: &mut Context) -> EndpointResult<Self::Future> {
        let a = self.0.apply(ctx)?;
        let _ = self.1.apply(ctx)?;
        Ok(a)
    }
}


#[allow(missing_docs)]
pub struct Map<E, F>(pub(crate) E, pub(crate) F);

impl<E, F, R> Endpoint for Map<E, F>
where
    E: Endpoint,
    F: FnOnce(E::Item) -> R,
{
    type Item = R;
    type Future = future::Map<E::Future, F>;

    fn apply(self, ctx: &mut Context) -> EndpointResult<Self::Future> {
        let Map(endpoint, f) = self;
        let fut = endpoint.apply(ctx)?;
        Ok(fut.map(f))
    }
}


#[allow(missing_docs)]
pub struct AndThen<E, F>(pub(crate) E, pub(crate) F);

impl<E, F, Fut, R> Endpoint for AndThen<E, F>
where
    E: Endpoint,
    F: FnOnce(E::Item) -> Fut,
    Fut: IntoFuture<Item = R, Error = FinchersError>,
{
    type Item = R;
    type Future = future::AndThen<E::Future, Fut, F>;

    fn apply(self, ctx: &mut Context) -> EndpointResult<Self::Future> {
        let AndThen(endpoint, f) = self;
        let fut = endpoint.apply(ctx)?;
        Ok(fut.and_then(f))
    }
}


#[allow(missing_docs)]
pub struct Or<E1, E2>(pub(crate) E1, pub(crate) E2);

impl<E1, E2> Endpoint for Or<E1, E2>
where
    E1: Endpoint,
    E2: Endpoint,
{
    type Item = Either2<E1::Item, E2::Item>;
    type Future = Either2<E1::Future, E2::Future>;

    fn apply(self, ctx: &mut Context) -> EndpointResult<Self::Future> {
        let mut ctx1 = ctx.clone();
        if let Ok(f) = self.0.apply(&mut ctx1) {
            *ctx = ctx1;
            return Ok(Either2::E1(f));
        }

        self.1.apply(ctx).map(Either2::E2)
    }
}


#[doc(hidden)]
pub trait IntoOr {
    type Out;
    fn into_either(self) -> Self::Out;
}

impl<E1, E2> IntoOr for (E1, E2)
where
    E1: Endpoint,
    E2: Endpoint,
{
    type Out = Or<E1, E2>;
    fn into_either(self) -> Self::Out {
        let (e1, e2) = self;
        Or(e1, e2)
    }
}
