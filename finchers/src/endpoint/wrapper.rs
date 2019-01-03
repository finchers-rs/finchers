//! Built-in wrappers.

mod after_apply;
mod before_apply;
mod or_reject;
mod recover;

pub use self::after_apply::{after_apply, AfterApply};
pub use self::before_apply::{before_apply, BeforeApply};
pub use self::or_reject::{or_reject, or_reject_with, OrReject, OrRejectWith};
pub use self::recover::{recover, Recover};

use crate::common::Tuple;
use crate::endpoint::Endpoint;

/// A trait representing the conversion of an endpoint to another endpoint.
pub trait Wrapper<Bd, E: Endpoint<Bd>> {
    /// The inner type of converted `Endpoint`.
    type Output: Tuple;

    /// The type of converted `Endpoint`.
    type Endpoint: Endpoint<Bd, Output = Self::Output>;

    /// Performs conversion from the provided endpoint into `Self::Endpoint`.
    fn wrap(self, endpoint: E) -> Self::Endpoint;
}
