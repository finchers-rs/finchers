//! Extensions for `Result`

mod and_then;
mod err;
mod err_into;
mod map_err;
mod map_ok;
mod ok;
mod or_else;

pub use self::and_then::AndThen;
pub use self::err::{err, Err};
pub use self::err_into::ErrInto;
pub use self::map_err::MapErr;
pub use self::map_ok::MapOk;
pub use self::ok::{ok, Ok};
pub use self::or_else::OrElse;

use crate::endpoint::{assert_output, EndpointBase};
use crate::generic::One;
use std::marker::PhantomData;

/// A helper trait enforcing that the type is `Result`.
pub trait IsResult: sealed::Sealed {
    /// The type of success value.
    type Ok;

    /// The type of error value.
    type Err;
}

impl<T, E> IsResult for One<Result<T, E>> {
    type Ok = T;
    type Err = E;
}

mod sealed {
    use crate::generic::One;

    pub trait Sealed {}

    impl<T, E> Sealed for One<Result<T, E>> {}
}

/// A set of extension methods which is available when the output value is a `Result`.
pub trait EndpointResultExt<A, B>: EndpointBase<Output = One<Result<A, B>>> + Sized {
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
        assert_output::<_, One<Result<U, B>>>(MapOk { endpoint: self, f })
    }

    /// Create an endpoint which will map the error value to a new type with given function.
    fn map_err<F, U>(self, f: F) -> MapErr<Self, F>
    where
        F: FnOnce(B) -> U + Clone,
    {
        assert_output::<_, One<Result<A, U>>>(MapErr { endpoint: self, f })
    }

    /// Create an endpoint which will convert the error value to a new type.
    fn err_into<U>(self) -> ErrInto<Self, U>
    where
        B: Into<U>,
    {
        assert_output::<_, One<Result<A, U>>>(ErrInto {
            endpoint: self,
            _marker: PhantomData,
        })
    }

    /// Create an endpoint which will map the successful value to a `Result` with given function.
    fn and_then<F, U>(self, f: F) -> AndThen<Self, F>
    where
        F: FnOnce(A) -> Result<U, B> + Clone,
    {
        assert_output::<_, One<Result<U, B>>>(AndThen { endpoint: self, f })
    }

    /// Create an endpoint which will map the error value to a `Result` with given function.
    fn or_else<F, U>(self, f: F) -> OrElse<Self, F>
    where
        F: FnOnce(B) -> Result<A, U> + Clone,
    {
        assert_output::<_, One<Result<A, U>>>(OrElse { endpoint: self, f })
    }
}

impl<E, A, B> EndpointResultExt<A, B> for E where E: EndpointBase<Output = One<Result<A, B>>> {}
