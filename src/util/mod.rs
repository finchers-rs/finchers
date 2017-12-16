//! Utilities

pub mod either;

use std::{error, fmt};
use context::Context;
use response::{Responder, Response};


/// A type represents the never-returned errors.
#[derive(Debug)]
pub enum NoReturn {}

impl fmt::Display for NoReturn {
    fn fmt(&self, _: &mut fmt::Formatter) -> fmt::Result {
        unreachable!()
    }
}

impl error::Error for NoReturn {
    fn description(&self) -> &str {
        unreachable!()
    }
}

impl Responder for NoReturn {
    fn respond_to(&mut self, _: &mut Context) -> Response {
        unreachable!()
    }
}
