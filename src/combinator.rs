use std::borrow::Cow;
use std::marker::PhantomData;
use std::str::FromStr;
use futures::Future;
use futures::future::{self, ok, FutureResult};

use endpoint::{Context, Endpoint};
use request::Request;

pub mod join {
    use endpoint::{Context, Endpoint};
    use request::Request;
    use futures::Future;
    use futures::future::{Join, Join3, Join4, Join5};

    // TODO: use macro to derive implementations.
    impl<A, B> Endpoint for (A, B)
    where
        A: Endpoint,
        B: Endpoint<Error = A::Error>,
    {
        type Item = (A::Item, B::Item);
        type Error = A::Error;
        type Future = Join<A::Future, B::Future>;
        fn apply(self, req: &Request, ctx: &mut Context) -> Result<Self::Future, ()> {
            let a = self.0.apply(req, ctx)?;
            let b = self.1.apply(req, ctx)?;
            Ok(a.join(b))
        }
    }

    impl<A, B, C> Endpoint for (A, B, C)
    where
        A: Endpoint,
        B: Endpoint<Error = A::Error>,
        C: Endpoint<Error = A::Error>,
    {
        type Item = (A::Item, B::Item, C::Item);
        type Error = A::Error;
        type Future = Join3<A::Future, B::Future, C::Future>;
        fn apply(self, req: &Request, ctx: &mut Context) -> Result<Self::Future, ()> {
            let a = self.0.apply(req, ctx)?;
            let b = self.1.apply(req, ctx)?;
            let c = self.2.apply(req, ctx)?;
            Ok(a.join3(b, c))
        }
    }

    impl<A, B, C, D> Endpoint for (A, B, C, D)
    where
        A: Endpoint,
        B: Endpoint<Error = A::Error>,
        C: Endpoint<Error = A::Error>,
        D: Endpoint<Error = A::Error>,
    {
        type Item = (A::Item, B::Item, C::Item, D::Item);
        type Error = A::Error;
        type Future = Join4<A::Future, B::Future, C::Future, D::Future>;
        fn apply(self, req: &Request, ctx: &mut Context) -> Result<Self::Future, ()> {
            let a = self.0.apply(req, ctx)?;
            let b = self.1.apply(req, ctx)?;
            let c = self.2.apply(req, ctx)?;
            let d = self.3.apply(req, ctx)?;
            Ok(a.join4(b, c, d))
        }
    }

    impl<A, B, C, D, E> Endpoint for (A, B, C, D, E)
    where
        A: Endpoint,
        B: Endpoint<Error = A::Error>,
        C: Endpoint<Error = A::Error>,
        D: Endpoint<Error = A::Error>,
        E: Endpoint<Error = A::Error>,
    {
        type Item = (A::Item, B::Item, C::Item, D::Item, E::Item);
        type Error = A::Error;
        type Future = Join5<A::Future, B::Future, C::Future, D::Future, E::Future>;
        fn apply(self, req: &Request, ctx: &mut Context) -> Result<Self::Future, ()> {
            let a = self.0.apply(req, ctx)?;
            let b = self.1.apply(req, ctx)?;
            let c = self.2.apply(req, ctx)?;
            let d = self.3.apply(req, ctx)?;
            let e = self.4.apply(req, ctx)?;
            Ok(a.join5(b, c, d, e))
        }
    }
}


pub struct With<E1, E2>(pub(crate) E1, pub(crate) E2);

impl<E1, E2> Endpoint for With<E1, E2>
where
    E1: Endpoint,
    E2: Endpoint<Error = E1::Error>,
{
    type Item = E2::Item;
    type Error = E2::Error;
    type Future = E2::Future;

    fn apply(self, req: &Request, ctx: &mut Context) -> Result<Self::Future, ()> {
        let With(e1, e2) = self;
        e1.apply(req, ctx).and_then(|_| e2.apply(req, ctx))
    }
}


pub struct Map<E, F>(pub(crate) E, pub(crate) F);

impl<E, F, R> Endpoint for Map<E, F>
where
    E: Endpoint,
    F: FnOnce(E::Item) -> R,
{
    type Item = R;
    type Error = E::Error;
    type Future = future::Map<E::Future, F>;

    fn apply(self, req: &Request, ctx: &mut Context) -> Result<Self::Future, ()> {
        let f = self.0.apply(req, ctx)?;
        Ok(f.map(self.1))
    }
}


// --------------------------------------------------------------------------------------

impl<'a> Endpoint for &'a str {
    type Item = ();
    type Error = ();
    type Future = FutureResult<(), ()>;

    fn apply(self, _req: &Request, ctx: &mut Context) -> Result<Self::Future, ()> {
        match ctx.routes.get(0) {
            Some(s) if s == self => {}
            _ => return Err(()),
        }
        ctx.routes.pop_front();
        Ok(ok(()))
    }
}

impl Endpoint for String {
    type Item = ();
    type Error = ();
    type Future = FutureResult<(), ()>;

    fn apply(self, req: &Request, ctx: &mut Context) -> Result<Self::Future, ()> {
        (&self as &str).apply(req, ctx)
    }
}

impl<'a> Endpoint for Cow<'a, str> {
    type Item = ();
    type Error = ();
    type Future = FutureResult<(), ()>;

    fn apply(self, req: &Request, ctx: &mut Context) -> Result<Self::Future, ()> {
        (&self as &str).apply(req, ctx)
    }
}


pub struct Path<T>(PhantomData<fn(T) -> T>);

impl<T: FromStr> Endpoint for Path<T> {
    type Item = T;
    type Error = ();
    type Future = FutureResult<T, ()>;

    fn apply(self, _req: &Request, ctx: &mut Context) -> Result<Self::Future, ()> {
        let value: T = match ctx.routes.get(0).and_then(|s| s.parse().ok()) {
            Some(val) => val,
            _ => return Err(()),
        };
        ctx.routes.pop_front();
        Ok(ok(value))
    }
}

pub fn path<T: FromStr>() -> Path<T> {
    Path(PhantomData)
}


pub struct PathSeq<T>(PhantomData<fn(T) -> T>);

impl<T: FromStr> Endpoint for PathSeq<T> {
    type Item = Vec<T>;
    type Error = ();
    type Future = FutureResult<Vec<T>, ()>;

    fn apply(self, _req: &Request, ctx: &mut Context) -> Result<Self::Future, ()> {
        let seq = ctx.routes.iter().filter_map(|s| s.parse().ok()).collect();
        Ok(ok(seq))
    }
}

pub fn path_seq<T: FromStr>() -> PathSeq<T> {
    PathSeq(PhantomData)
}
