//! `Responder` layer

pub(crate) mod inner;
mod responder;

pub use self::responder::{ErrorResponder, IntoResponder, Responder, StringResponder};
