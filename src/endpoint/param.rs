use std::borrow::{Borrow, Cow};
use futures::future::IntoFuture;
use futures::future::FutureResult;
use super::Endpoint;
use input::Input;


#[derive(Debug)]
pub struct Param {
    name: Cow<'static, str>,
}

impl Endpoint for Param {
    type Item = String;
    type Error = ParamIsNotSet;
    type Future = FutureResult<String, ParamIsNotSet>;

    fn apply(&self, input: Input) -> Result<Self::Future, Input> {
        let value = if let Some(value) = input.params.get(self.name.borrow() as &str) {
            Some(Ok(value.to_owned()))
        } else {
            None
        };
        value.map(IntoFuture::into_future).ok_or(input)
    }
}


#[derive(Debug)]
pub struct ParamIsNotSet(Cow<'static, str>);

pub fn param<S: Into<Cow<'static, str>>>(name: S) -> Param {
    Param { name: name.into() }
}
