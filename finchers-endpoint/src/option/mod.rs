mod map_some;
mod ok_or_else;

pub use self::map_some::MapSome;
pub use self::ok_or_else::OkOrElse;

use common::assert_output;
use finchers_core::{Endpoint, IsOption};

pub trait EndpointOptionExt<T>: Endpoint<Output = Option<T>> + Sized {
    #[inline(always)]
    fn as_option<U>(self) -> Self
    where
        Self::Output: IsOption<Item = T>,
    {
        self
    }

    fn map_some<F, U>(self, f: F) -> MapSome<Self, F>
    where
        F: FnOnce(T) -> U + Clone + Send,
    {
        assert_output::<_, Option<U>>(self::map_some::new(self, f))
    }

    fn ok_or_else<F, U>(self, f: F) -> OkOrElse<Self, F>
    where
        F: FnOnce() -> U + Clone + Send,
    {
        assert_output::<_, Result<T, U>>(self::ok_or_else::new(self, f))
    }
}

impl<E, T> EndpointOptionExt<T> for E
where
    E: Endpoint<Output = Option<T>>,
{
}
