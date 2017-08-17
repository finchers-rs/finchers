use std::borrow::Cow;
use std::marker::PhantomData;
use std::str::FromStr;
use futures::{Future, Poll, Async};
use input::Input;
use super::Endpoint;


#[derive(Debug)]
pub struct MatchPath<E: Endpoint> {
    name: Cow<'static, str>,
    endpoint: E,
}

impl<E: Endpoint> Endpoint for MatchPath<E> {
    type Item = E::Item;
    type Error = E::Error;
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


#[derive(Debug)]
pub struct PathEnd<E: Endpoint> {
    endpoint: E,
}

impl<E: Endpoint> Endpoint for PathEnd<E> {
    type Item = E::Item;
    type Error = E::Error;
    type Future = E::Future;

    fn apply(&self, input: Input) -> Result<Self::Future, Input> {
        if input.routes.len() > 0 {
            return Err(input);
        }
        self.endpoint.apply(input)
    }
}


#[derive(Debug)]
pub struct Path<T: FromStr, E: Endpoint> {
    endpoint: E,
    _marker: PhantomData<T>,
}

impl<T: FromStr, E: Endpoint> Endpoint for Path<T, E> {
    type Item = (T, E::Item);
    type Error = E::Error;
    type Future = PathFuture<T, E::Future>;

    fn apply(&self, mut input: Input) -> Result<Self::Future, Input> {
        let value: T = match input.routes.get(0).and_then(|s| s.parse().ok()) {
            Some(v) => v,
            None => return Err(input),
        };
        input.routes = input.routes.into_iter().skip(1).collect();
        Ok(PathFuture {
            value: Some(value),
            future: self.endpoint.apply(input)?,
        })
    }
}

pub struct PathFuture<T, F: Future> {
    value: Option<T>,
    future: F,
}

impl<T, F: Future> Future for PathFuture<T, F> {
    type Item = (T, F::Item);
    type Error = F::Error;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let rhs = try_ready!(self.future.poll());
        let lhs = self.value.take().expect("cannot resolve twice");
        Ok(Async::Ready((lhs, rhs)))
    }
}

pub fn value<T: FromStr, E: Endpoint>(endpoint: E) -> Path<T, E> {
    Path {
        endpoint,
        _marker: PhantomData,
    }
}

pub fn path<S: Into<Cow<'static, str>>, E: Endpoint>(name: S, endpoint: E) -> MatchPath<E> {
    MatchPath {
        name: name.into(),
        endpoint,
    }
}

pub fn path_end<E: Endpoint>(endpoint: E) -> PathEnd<E> {
    PathEnd { endpoint }
}
