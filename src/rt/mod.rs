//! Runtime support for Finchers, which supports serving asynchronous HTTP services.

pub mod local;

mod app;
mod server;

pub use self::server::{launch, LaunchResult};

/// A type alias represents endpoints which can be used as an HTTP application.
pub trait AppEndpoint: sealed::Sealed {}

mod sealed {
    use super::AppEndpoint;

    use futures_core::future::TryFuture;
    use std::mem::PinMut;

    use endpoint::Endpoint;
    use error::Error;
    use input::{Cursor, Input};
    use output::Responder;

    pub trait Sealed: Send + Sync + 'static {
        type Output: Responder;
        type Future: TryFuture<Ok = Self::Output, Error = Error> + Send + 'static;

        fn apply(
            &self,
            input: PinMut<'_, Input>,
            cursor: Cursor<'c>,
        ) -> Option<(Self::Future, Cursor<'c>)>;
    }

    impl<E> Sealed for E
    where
        E: Endpoint + Send + Sync + 'static,
        E::Output: Responder,
        E::Future: Send + 'static,
    {
        type Output = E::Output;
        type Future = E::Future;

        #[inline(always)]
        fn apply(
            &self,
            input: PinMut<'_, Input>,
            cursor: Cursor<'c>,
        ) -> Option<(Self::Future, Cursor<'c>)> {
            Endpoint::apply(self, input, cursor)
        }
    }

    impl<E> AppEndpoint for E
    where
        E: Endpoint + Send + Sync + 'static,
        E::Output: Responder,
        E::Future: Send + 'static,
    {}
}
