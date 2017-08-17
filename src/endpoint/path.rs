use std::borrow::Cow;
use input::Input;
use super::Endpoint;


#[derive(Debug)]
pub struct Path<E: Endpoint> {
    name: Cow<'static, str>,
    endpoint: E,
}

impl<E: Endpoint> Endpoint for Path<E> {
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

pub fn path<S: Into<Cow<'static, str>>, E: Endpoint>(name: S, endpoint: E) -> Path<E> {
    Path {
        name: name.into(),
        endpoint,
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

pub fn path_end<E: Endpoint>(endpoint: E) -> PathEnd<E> {
    PathEnd { endpoint }
}
