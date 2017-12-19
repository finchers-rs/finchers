use std::borrow::Cow;
use std::iter::FromIterator;
use std::marker::PhantomData;
use std::str::FromStr;

use endpoint::{Endpoint, EndpointContext, EndpointError, IntoEndpoint};
use task::{ok, TaskResult};



#[derive(Debug, Clone)]
pub struct PathSegment<'a, E>(Cow<'a, str>, PhantomData<fn() -> E>);

impl<'a, E> Endpoint for PathSegment<'a, E> {
    type Item = ();
    type Error = E;
    type Task = TaskResult<Self::Item, Self::Error>;

    fn apply(&self, ctx: &mut EndpointContext) -> Result<Self::Task, EndpointError> {
        if !ctx.next_segment().map(|s| s == self.0).unwrap_or(false) {
            return Err(EndpointError::Skipped);
        }
        Ok(ok(()))
    }
}

impl<'a, E> IntoEndpoint<(), E> for &'a str {
    type Endpoint = PathSegment<'a, E>;
    fn into_endpoint(self) -> Self::Endpoint {
        segment(self)
    }
}

impl<E> IntoEndpoint<(), E> for String {
    type Endpoint = PathSegment<'static, E>;
    fn into_endpoint(self) -> Self::Endpoint {
        segment(self)
    }
}

impl<'a, E> IntoEndpoint<(), E> for Cow<'a, str> {
    type Endpoint = PathSegment<'a, E>;
    fn into_endpoint(self) -> Self::Endpoint {
        segment(self)
    }
}


#[inline(always)]
pub fn segment<'a, T: 'a + Into<Cow<'a, str>>, E>(segment: T) -> PathSegment<'a, E> {
    PathSegment(segment.into(), PhantomData)
}



#[derive(Debug)]
pub struct PathParam<T, E>(PhantomData<fn() -> (T, E)>);

impl<T, E> Copy for PathParam<T, E> {}

impl<T, E> Clone for PathParam<T, E> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: FromStr, E> Endpoint for PathParam<T, E> {
    type Item = T;
    type Error = E;
    type Task = TaskResult<Self::Item, Self::Error>;

    fn apply(&self, ctx: &mut EndpointContext) -> Result<Self::Task, EndpointError> {
        match ctx.next_segment().map(|s| s.parse()) {
            Some(Ok(value)) => Ok(ok(value)),
            _ => return Err(EndpointError::TypeMismatch),
        }
    }
}


pub fn param<T: FromStr, E>() -> PathParam<T, E> {
    PathParam(PhantomData)
}



#[derive(Debug)]
pub struct PathParams<I, T, E>(PhantomData<fn() -> (I, T, E)>);

impl<I, T, E> Copy for PathParams<I, T, E> {}

impl<I, T, E> Clone for PathParams<I, T, E> {
    fn clone(&self) -> Self {
        *self
    }
}


impl<I, T, E> Endpoint for PathParams<I, T, E>
where
    I: FromIterator<T> + Default,
    T: FromStr,
{
    type Item = I;
    type Error = E;
    type Task = TaskResult<Self::Item, Self::Error>;

    fn apply(&self, ctx: &mut EndpointContext) -> Result<Self::Task, EndpointError> {
        match ctx.collect_remaining_segments() {
            Some(Ok(seq)) => Ok(ok(seq)),
            Some(Err(_)) => Err(EndpointError::TypeMismatch),
            None => Ok(ok(Default::default())),
        }
    }
}


pub fn params<I, T, E>() -> PathParams<I, T, E>
where
    I: FromIterator<T>,
    T: FromStr,
{
    PathParams(PhantomData)
}
