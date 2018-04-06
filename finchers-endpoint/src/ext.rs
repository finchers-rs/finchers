use super::*;
use endpoint::{Endpoint, IntoEndpoint};
use finchers_core::Error;
use futures::IntoFuture;

pub trait EndpointExt: Endpoint {
    fn join<E>(self, e: E) -> Join<Self, E::Endpoint>
    where
        E: IntoEndpoint,
        Self: Sized,
    {
        assert_endpoint::<_, (Self::Item, <E::Endpoint as Endpoint>::Item)>(join::join(self, e))
    }

    fn with<E>(self, e: E) -> With<Self, E::Endpoint>
    where
        E: IntoEndpoint,
        Self: Sized,
    {
        assert_endpoint::<_, E::Item>(with::with(self, e))
    }

    fn skip<E>(self, e: E) -> Skip<Self, E::Endpoint>
    where
        E: IntoEndpoint,
        Self: Sized,
    {
        assert_endpoint::<_, Self::Item>(skip::skip(self, e))
    }

    fn or<E>(self, e: E) -> Or<Self, E::Endpoint>
    where
        E: IntoEndpoint<Item = Self::Item>,
        Self: Sized,
    {
        assert_endpoint::<_, Self::Item>(or::or(self, e))
    }

    fn map<F, T>(self, f: F) -> map::Map<Self, F>
    where
        F: Fn(Self::Item) -> T,
        Self: Sized,
    {
        assert_endpoint::<_, T>(map::map(self, f))
    }

    fn and_then<F, R>(self, f: F) -> AndThen<Self, F>
    where
        F: Fn(Self::Item) -> R,
        R: IntoFuture,
        R::Error: Into<Error>,
        Self: Sized,
    {
        assert_endpoint::<_, R::Item>(and_then::and_then(self, f))
    }
}

impl<E: Endpoint + ?Sized> EndpointExt for E {}

#[inline]
fn assert_endpoint<E, T>(endpoint: E) -> E
where
    E: Endpoint<Item = T>,
{
    endpoint
}
