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
mod string;

// re-exports
pub use apply::{apply, Apply};
pub use endpoint::Endpoint;
pub use error::{Error, HttpError};
pub use input::Input;
pub use output::Output;
pub use string::BytesString;
pub use task::Task;
