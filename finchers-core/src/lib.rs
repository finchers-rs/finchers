#![doc(html_url = "https://docs.rs/finchers-core/0.11.0")]
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![warn(warnings)]

extern crate bytes;
#[macro_use]
extern crate futures;
extern crate http;
extern crate mime;
#[macro_use]
extern crate scoped_tls;
extern crate either;

#[cfg(feature = "from_hyper")]
extern crate hyper;

pub mod endpoint;
pub mod error;
pub mod input;
pub mod output;
pub mod task;

mod apply;

// re-exports
pub use apply::{apply, Apply};
pub use endpoint::Endpoint;
pub use error::{Error, HttpError};
pub use input::Input;
pub use output::{Output, Responder};
pub use task::Task;
