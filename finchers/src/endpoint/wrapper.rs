//! Built-in wrappers.

mod after_apply;
mod and_then;
mod before_apply;
mod map;
mod or_reject;
mod recover;

pub use self::after_apply::{after_apply, AfterApply};
pub use self::and_then::{and_then, AndThen};
pub use self::before_apply::{before_apply, BeforeApply};
pub use self::map::{map, Map};
pub use self::or_reject::{or_reject, or_reject_with, OrReject, OrRejectWith};
pub use self::recover::{recover, Recover};

use crate::common::{Func, Tuple};
use crate::endpoint::Endpoint;
use crate::future::EndpointFuture;

/// A trait representing the conversion of an endpoint to another endpoint.
pub trait Wrapper<E: Endpoint> {
    /// The inner type of converted `Endpoint`.
    type Output: Tuple;

    /// The type of converted `Endpoint`.
    type Endpoint: Endpoint<Output = Self::Output>;

    /// Performs conversion from the provided endpoint into `Self::Endpoint`.
    fn wrap(self, endpoint: E) -> Self::Endpoint;
}

/// A set of extension methods for using built-in `Wrapper`s.
pub trait EndpointWrapExt: Endpoint + Sized {
    #[allow(missing_docs)]
    fn map<F>(self, f: F) -> <Map<Self::Output, F> as Wrapper<Self>>::Endpoint
    where
        F: Func<Self::Output> + Clone,
    {
        self.wrap(map(f))
    }

    #[allow(missing_docs)]
    fn and_then<F>(self, f: F) -> <AndThen<Self::Output, F> as Wrapper<Self>>::Endpoint
    where
        F: Func<Self::Output> + Clone,
        F::Out: EndpointFuture,
    {
        self.wrap(and_then(f))
    }
}

impl<E: Endpoint> EndpointWrapExt for E {}
