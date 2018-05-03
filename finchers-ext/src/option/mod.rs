//! Extensions for `Option`

mod map_some;
mod ok_or_else;

pub use self::map_some::MapSome;
pub use self::ok_or_else::OkOrElse;

use common::assert_output;
use finchers_core::Endpoint;

/// A set of extension methods which is available when the output is value is an `Option`.
pub trait EndpointOptionExt<T>: Endpoint<Output = Option<T>> + Sized {
    /// Create an endpoint which will map the value to a new type with given function.
    fn map_some<F, U>(self, f: F) -> MapSome<Self, F>
    where
        F: FnOnce(T) -> U + Clone + Send + Sync,
    {
        assert_output::<_, Option<U>>(self::map_some::new(self, f))
    }

    /// Create an endpoint which will transform the returned value into a `Result`.
    fn ok_or_else<F, U>(self, f: F) -> OkOrElse<Self, F>
    where
        F: FnOnce() -> U + Clone + Send + Sync,
    {
        assert_output::<_, Result<T, U>>(self::ok_or_else::new(self, f))
    }
}

impl<E, T> EndpointOptionExt<T> for E
where
    E: Endpoint<Output = Option<T>>,
{
}
