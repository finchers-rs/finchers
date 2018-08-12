//! Runtime support for Finchers, which supports serving asynchronous HTTP services.

pub mod app;
pub mod server;

/// A type alias represents endpoints which can be used as an HTTP application.
pub trait AppEndpoint: sealed::Sealed {}

mod sealed {
    use super::AppEndpoint;

    use futures_core::future::TryFuture;
    use std::mem::PinMut;

    use finchers_core::endpoint::Endpoint;
    use finchers_core::error::Error;
    use finchers_core::input::{Cursor, Input};
    use finchers_core::output::Responder;

    pub trait Sealed: Send + Sync + 'static {
        type Output: Responder;
        type Future: TryFuture<Ok = Self::Output, Error = Error> + Send + 'static;

        fn apply(&self, input: PinMut<Input>) -> Option<Self::Future>;
    }

    impl<E> Sealed for E
    where
        E: Endpoint + Send + Sync + 'static,
        E::Output: Responder,
        E::Future: Send + 'static,
    {
        type Output = E::Output;
        type Future = E::Future;

        fn apply(&self, input: PinMut<Input>) -> Option<Self::Future> {
            let cursor = unsafe {
                let path = &*(input.uri().path() as *const str);
                Cursor::new(path)
            };
            Endpoint::apply(self, input, cursor).map(|(future, _rest)| future)
        }
    }

    impl<E> AppEndpoint for E
    where
        E: Endpoint + Send + Sync + 'static,
        E::Output: Responder,
        E::Future: Send + 'static,
    {}
}
