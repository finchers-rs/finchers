#![cfg_attr(feature = "nightly", feature(try_trait))]

extern crate bytes;
extern crate either;
#[macro_use]
extern crate futures;
extern crate http;
extern crate mime;
#[macro_use]
extern crate scoped_tls;
extern crate failure;

#[cfg(feature = "from_hyper")]
extern crate hyper;

pub mod endpoint;
pub mod error;
pub mod input;
pub mod outcome;
pub mod output;

mod apply;
mod never;
mod option;
mod result;

// re-exports
pub use apply::{apply, Apply};
pub use endpoint::Endpoint;
pub use error::{Error, HttpError};
pub use input::Input;
pub use never::Never;
pub use option::IsOption;
pub use outcome::Outcome;
pub use output::{Output, Responder};
pub use result::IsResult;
