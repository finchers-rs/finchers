use std::borrow::Cow;
use futures::{Future, IntoFuture};
use futures::future::FutureResult;
use url::form_urlencoded;

use input::Input;

pub trait Endpoint {
    type Future: Future;
    fn apply(&self, input: Input) -> Result<Self::Future, Input>;
}

impl<F, R> Endpoint for F
where
    F: Fn(Input) -> Result<R, Input>,
    R: IntoFuture,
{
    type Future = R::Future;

    fn apply(&self, input: Input) -> Result<Self::Future, Input> {
        (*self)(input).map(IntoFuture::into_future)
    }
}


#[derive(Debug)]
pub struct Param {
    name: Cow<'static, str>,
}

impl Endpoint for Param {
    type Future = FutureResult<String, ParamIsNotSet>;

    fn apply(&self, input: Input) -> Result<Self::Future, Input> {
        Ok(
            input
                .req
                .query()
                .and_then(|query| {
                    form_urlencoded::parse(query.as_bytes())
                        .find(|&(ref k, _)| k == &self.name)
                        .map(|(_, v)| v.into_owned())
                })
                .ok_or(ParamIsNotSet(self.name.clone()))
                .into_future(),
        )
    }
}

#[derive(Debug)]
pub struct ParamIsNotSet(Cow<'static, str>);

pub fn param<S: Into<Cow<'static, str>>>(name: S) -> Param {
    Param { name: name.into() }
}



#[derive(Debug)]
pub struct PathEnd<E: Endpoint> {
    endpoint: E,
}

impl<E: Endpoint> Endpoint for PathEnd<E> {
    type Future = E::Future;

    fn apply(&self, input: Input) -> Result<Self::Future, Input> {
        if input.routes.len() > 0 {
            return Err(input);
        }
        self.endpoint.apply(input)
    }
}

pub fn path_end<E: Endpoint>(endpoint: E) -> PathEnd<E> {
    PathEnd { endpoint }
}


#[derive(Debug)]
pub struct Path<E: Endpoint> {
    name: Cow<'static, str>,
    endpoint: E,
}

impl<E: Endpoint> Endpoint for Path<E> {
    type Future = E::Future;

    fn apply(&self, mut input: Input) -> Result<Self::Future, Input> {
        let is_matched = input
            .routes
            .get(0)
            .map(|route| route == &self.name)
            .unwrap_or(false);
        if !is_matched {
            return Err(input);
        }

        input.routes = input.routes.into_iter().skip(1).collect();
        self.endpoint.apply(input)
    }
}

pub fn path<S: Into<Cow<'static, str>>, E: Endpoint>(name: S, endpoint: E) -> Path<E> {
    Path {
        name: name.into(),
        endpoint,
    }
}
