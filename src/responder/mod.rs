//! `Responder` layer

mod context;
mod responder;

pub use self::context::{respond, ResponderContext};
pub use self::responder::{ErrorResponder, IntoResponder, Responder, StringResponder};
