//! Extensions for `Result`

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

use crate::endpoint::{assert_output, EndpointBase};

/// A helper trait enforcing that the type is `Result`.
pub trait IsResult: sealed::Sealed {
    /// The type of success value.
    type Ok;

    /// The type of error value.
    type Err;

    /// Consume itself and get the value of `Result`.
    fn into_result(self) -> Result<Self::Ok, Self::Err>;
}

impl<T, E> IsResult for Result<T, E> {
    type Ok = T;
    type Err = E;

    #[inline(always)]
    fn into_result(self) -> Result<Self::Ok, Self::Err> {
        self
    }
}

mod sealed {
    pub trait Sealed {}

    impl<T, E> Sealed for Result<T, E> {}
}

/// A set of extension methods which is available when the output value is a `Result`.
pub trait EndpointResultExt<A, B>: EndpointBase<Output = Result<A, B>> + Sized {
    /// Annotate that the successful type of associated type `Output` is equal to `T`.
    #[inline(always)]
    fn as_ok<T>(self) -> Self
    where
        Self::Output: IsResult<Ok = T>,
    {
        self
    }

    /// Annotate that the error type of associated type `Output` is equal to `E`.
    #[inline(always)]
    fn as_err<E>(self) -> Self
    where
        Self::Output: IsResult<Err = E>,
    {
        self
    }

    /// Create an endpoint which will map the successful value to a new type with given function.
    fn map_ok<F, U>(self, f: F) -> MapOk<Self, F>
    where
        F: FnOnce(A) -> U + Clone,
    {
        assert_output::<_, Result<U, B>>(self::map_ok::new(self, f))
    }

    /// Create an endpoint which will map the error value to a new type with given function.
    fn map_err<F, U>(self, f: F) -> MapErr<Self, F>
    where
        F: FnOnce(B) -> U + Clone,
    {
        assert_output::<_, Result<A, U>>(self::map_err::new(self, f))
    }

    /// Create an endpoint which will convert the error value to a new type.
    fn err_into<U>(self) -> ErrInto<Self, U>
    where
        B: Into<U>,
    {
        assert_output::<_, Result<A, U>>(self::err_into::new(self))
    }

    /// Create an endpoint which will map the successful value to a `Result` with given function.
    fn and_then<F, U>(self, f: F) -> AndThen<Self, F>
    where
        F: FnOnce(A) -> Result<U, B> + Clone,
    {
        assert_output::<_, Result<U, B>>(self::and_then::new(self, f))
    }

    /// Create an endpoint which will map the error value to a `Result` with given function.
    fn or_else<F, U>(self, f: F) -> OrElse<Self, F>
    where
        F: FnOnce(B) -> Result<A, U> + Clone,
    {
        assert_output::<_, Result<A, U>>(self::or_else::new(self, f))
    }
}

impl<E, A, B> EndpointResultExt<A, B> for E where E: EndpointBase<Output = Result<A, B>> {}
