//! Built-in wrappers.

mod after_apply;
mod before_apply;

pub use self::after_apply::{after_apply, AfterApply};
pub use self::before_apply::{before_apply, BeforeApply};

use super::Endpoint;
use crate::common::Tuple;

/// A trait representing the conversion of an endpoint to another endpoint.
pub trait Wrapper<'a, E: Endpoint<'a>> {
    /// The inner type of converted `Endpoint`.
    type Output: Tuple;

    /// The type of converted `Endpoint`.
    type Endpoint: Endpoint<'a, Output = Self::Output>;

    /// Performs conversion from the provided endpoint into `Self::Endpoint`.
    fn wrap(self, endpoint: E) -> Self::Endpoint;
}
