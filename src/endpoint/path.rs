//! Definition of endpoints to parse path segments

use std::borrow::Cow;
use std::iter::FromIterator;
use std::marker::PhantomData;
use std::str::FromStr;
use endpoint::{Endpoint, EndpointContext, IntoEndpoint};


#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct MatchPath<'a, E>(Cow<'a, str>, PhantomData<fn() -> E>);

impl<'a, E> Endpoint for MatchPath<'a, E> {
    type Item = ();
    type Error = E;
    type Task = Result<Self::Item, Self::Error>;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Task> {
        if !ctx.next_segment().map(|s| s == self.0).unwrap_or(false) {
            return None;
        }
        Some(Ok(()))
    }
}

impl<'a, E> IntoEndpoint<(), E> for &'a str {
    type Endpoint = MatchPath<'a, E>;
    fn into_endpoint(self) -> Self::Endpoint {
        MatchPath(self.into(), PhantomData)
    }
}

impl<E> IntoEndpoint<(), E> for String {
    type Endpoint = MatchPath<'static, E>;
    fn into_endpoint(self) -> Self::Endpoint {
        MatchPath(self.into(), PhantomData)
    }
}

impl<'a, E> IntoEndpoint<(), E> for Cow<'a, str> {
    type Endpoint = MatchPath<'a, E>;
    fn into_endpoint(self) -> Self::Endpoint {
        MatchPath(self.into(), PhantomData)
    }
}


/// Create an endpoint which represents a path element
pub fn path<T: FromStr, E>() -> ExtractPath<T, E> {
    ExtractPath(PhantomData)
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct ExtractPath<T, E>(PhantomData<fn() -> (T, E)>);

impl<T, E> Endpoint for ExtractPath<T, E>
where
    T: FromStr,
    E: From<T::Err>,
{
    type Item = T;
    type Error = E;
    type Task = Result<Self::Item, Self::Error>;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Task> {
        match ctx.next_segment().map(|s| s.parse()) {
            Some(res) => Some(res.map_err(Into::into)),
            _ => return None,
        }
    }
}



/// Create an endpoint which represents a path element
pub fn paths<I, T, E>() -> ExtractPaths<I, T, E>
where
    I: FromIterator<T>,
    T: FromStr,
{
    ExtractPaths(PhantomData)
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct ExtractPaths<I, T, E>(PhantomData<fn() -> (I, T, E)>);

impl<I, T, E> Endpoint for ExtractPaths<I, T, E>
where
    I: FromIterator<T>,
    T: FromStr,
    E: From<T::Err>,
{
    type Item = I;
    type Error = E;
    type Task = Result<Self::Item, Self::Error>;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Task> {
        match ctx.take_segments()
            .map(|s| s.map(|s| s.parse()).collect::<Result<_, _>>())
        {
            Some(res) => Some(res.map_err(Into::into)),
            _ => return None,
        }
    }
}
