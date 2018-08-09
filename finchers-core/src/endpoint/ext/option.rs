//! Extensions for `Option`

mod map_some;
mod ok_or_else;
mod some;

pub use self::map_some::MapSome;
pub use self::ok_or_else::OkOrElse;
pub use self::some::{some, Some};

use self::sealed::Sealed;
use crate::endpoint::{assert_output, EndpointBase};
use crate::generic::One;

/// A helper trait enforcing that the type is `Option`.
pub trait IsOption: Sealed {
    /// The type of inner value.
    type Item;
}

impl<T> IsOption for One<Option<T>> {
    type Item = T;
}

mod sealed {
    use crate::generic::One;
    pub trait Sealed {}

    impl<T> Sealed for One<Option<T>> {}
}

/// A set of extension methods which is available when the output is value is an `Option`.
pub trait EndpointOptionExt<T>: EndpointBase<Output = One<Option<T>>> + Sized {
    #[allow(missing_docs)]
    #[inline]
    fn as_some<U>(self) -> Self
    where
        Self::Output: IsOption<Item = U>,
    {
        self
    }

    /// Create an endpoint which will map the value to a new type with given function.
    fn map_some<F, U>(self, f: F) -> MapSome<Self, F>
    where
        F: FnOnce(T) -> U + Clone,
    {
        assert_output::<_, One<Option<U>>>(MapSome { endpoint: self, f })
    }

    /// Create an endpoint which will transform the returned value into a `Result`.
    fn ok_or_else<F, U>(self, f: F) -> OkOrElse<Self, F>
    where
        F: FnOnce() -> U + Clone,
    {
        assert_output::<_, One<Result<T, U>>>(OkOrElse { endpoint: self, f })
    }
}

impl<E, T> EndpointOptionExt<T> for E where E: EndpointBase<Output = One<Option<T>>> {}
