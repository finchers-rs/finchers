//! Extensions for `Option`

mod map_some;
mod ok_or_else;

pub use self::map_some::MapSome;
pub use self::ok_or_else::OkOrElse;

use self::sealed::Sealed;
use crate::endpoint::{assert_output, EndpointBase};

/// A helper trait enforcing that the type is `Option`.
pub trait IsOption: Sealed {
    /// The type of inner value.
    type Item;

    /// Consume itself and get the value of `Option`.
    fn into_option(self) -> Option<Self::Item>;
}

impl<T> IsOption for Option<T> {
    type Item = T;

    #[inline(always)]
    fn into_option(self) -> Option<Self::Item> {
        self
    }
}

mod sealed {
    pub trait Sealed {}

    impl<T> Sealed for Option<T> {}
}

/// A set of extension methods which is available when the output is value is an `Option`.
pub trait EndpointOptionExt<T>: EndpointBase<Output = Option<T>> + Sized {
    /// Create an endpoint which will map the value to a new type with given function.
    fn map_some<F, U>(self, f: F) -> MapSome<Self, F>
    where
        F: FnOnce(T) -> U + Clone,
    {
        assert_output::<_, Option<U>>(self::map_some::new(self, f))
    }

    /// Create an endpoint which will transform the returned value into a `Result`.
    fn ok_or_else<F, U>(self, f: F) -> OkOrElse<Self, F>
    where
        F: FnOnce() -> U + Clone,
    {
        assert_output::<_, Result<T, U>>(self::ok_or_else::new(self, f))
    }
}

impl<E, T> EndpointOptionExt<T> for E where E: EndpointBase<Output = Option<T>> {}
