#![cfg_attr(feature = "nightly", feature(try_trait))]

extern crate bytes;
extern crate either;
#[macro_use]
extern crate failure;
extern crate futures;
extern crate http;
extern crate mime;
#[macro_use]
extern crate scoped_tls;

#[cfg(feature = "hyper")]
extern crate hyper;

#[macro_use]
pub mod task;

pub mod endpoint;
pub mod error;
pub mod input;
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
pub use output::{Output, Responder};
pub use result::IsResult;
pub use task::Task;
