mod and_then;
mod err_into;
mod map_err;
mod map_ok;
mod or_else;

pub use self::and_then::AndThen;
pub use self::err_into::ErrInto;
pub use self::map_err::MapErr;
pub use self::map_ok::MapOk;
pub use self::or_else::OrElse;

use finchers_core::Endpoint;

pub trait EndpointResultExt<A, B>: Endpoint<Output = Result<A, B>> + Sized {
    fn map_ok<F, U>(self, f: F) -> MapOk<Self, F>
    where
        F: FnOnce(A) -> U + Clone + Send,
    {
        assert_endpoint::<_, U, B>(self::map_ok::new(self, f))
    }

    fn map_err<F, U>(self, f: F) -> MapErr<Self, F>
    where
        F: FnOnce(B) -> U + Clone + Send,
    {
        assert_endpoint::<_, A, U>(self::map_err::new(self, f))
    }

    fn and_then<F, U>(self, f: F) -> AndThen<Self, F>
    where
        F: FnOnce(A) -> Result<U, B> + Clone + Send,
    {
        assert_endpoint::<_, U, B>(self::and_then::new(self, f))
    }

    fn or_else<F, U>(self, f: F) -> OrElse<Self, F>
    where
        F: FnOnce(B) -> Result<A, U> + Clone + Send,
    {
        assert_endpoint::<_, A, U>(self::or_else::new(self, f))
    }

    fn err_into<U>(self) -> ErrInto<Self, U>
    where
        B: Into<U>,
    {
        assert_endpoint::<_, A, U>(self::err_into::new(self))
    }
}

impl<E, A, B> EndpointResultExt<A, B> for E
where
    E: Endpoint<Output = Result<A, B>>,
{
}

#[inline(always)]
fn assert_endpoint<E, A, B>(endpoint: E) -> E
where
    E: Endpoint<Output = Result<A, B>>,
{
    endpoint
}
