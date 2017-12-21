//! Definition of endpoints to parse query parameters

use std::borrow::Cow;
use std::marker::PhantomData;
use std::str::FromStr;

use endpoint::{Endpoint, EndpointContext, EndpointError};
use task::{result, TaskResult};

#[allow(missing_docs)]
#[derive(Debug)]
pub struct Param<'a, T, E>(Cow<'a, str>, PhantomData<fn() -> (T, E)>);

impl<'a, T, E> Endpoint for Param<'a, T, E>
where
    T: FromStr,
    E: From<T::Err>,
{
    type Item = T;
    type Error = E;
    type Task = TaskResult<Self::Item, Self::Error>;

    fn apply(&self, ctx: &mut EndpointContext) -> Result<Self::Task, EndpointError> {
        ctx.find_param(&*self.0)
            .and_then(|s| s.get(0).map(|s| s.parse().map_err(Into::into)))
            .map(result)
            .ok_or(EndpointError::Skipped)
    }
}

/// Create an endpoint matches a query parameter named `name`
pub fn param<T, E>(name: &'static str) -> Param<T, E>
where
    T: FromStr,
    E: From<T::Err>,
{
    Param(name.into(), PhantomData)
}
