#![allow(missing_docs)]

mod context;
mod responder;

pub use self::context::ResponderContext;
pub use self::responder::{respond, IntoResponder, Responder};
