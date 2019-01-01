#![allow(clippy::type_complexity)]

use crate::common::{Combine, Tuple};
use crate::endpoint::{ApplyContext, ApplyResult, Endpoint, IntoEndpoint};
use crate::error::Error;
use crate::future::{Context, EndpointFuture, MaybeDone, Poll};

#[allow(missing_docs)]
#[derive(Copy, Clone, Debug)]
pub struct And<E1, E2> {
    pub(super) e1: E1,
    pub(super) e2: E2,
}

impl<E1, E2> Endpoint for And<E1, E2>
where
    E1: Endpoint,
    E2: Endpoint,
    E1::Output: Combine<E2::Output>,
{
    type Output = <E1::Output as Combine<E2::Output>>::Out;
    type Future = AndFuture<E1::Future, E2::Future>;

    fn apply(&self, ecx: &mut ApplyContext<'_>) -> ApplyResult<Self::Future> {
        Ok(AndFuture {
            f1: self.e1.apply(ecx).map(MaybeDone::Pending)?,
            f2: self.e2.apply(ecx).map(MaybeDone::Pending)?,
        })
    }
}

#[allow(missing_debug_implementations)]
pub struct AndFuture<F1, F2>
where
    F1: EndpointFuture,
    F2: EndpointFuture,
{
    f1: MaybeDone<F1>,
    f2: MaybeDone<F2>,
}

impl<F1, F2> EndpointFuture for AndFuture<F1, F2>
where
    F1: EndpointFuture,
    F2: EndpointFuture,
    F1::Output: Combine<F2::Output>,
    F2::Output: Tuple,
{
    type Output = <F1::Output as Combine<F2::Output>>::Out;

    fn poll_endpoint(&mut self, cx: &mut Context<'_>) -> Poll<Self::Output, Error> {
        futures::try_ready!(self.f1.poll_endpoint(cx));
        futures::try_ready!(self.f2.poll_endpoint(cx));
        let v1 = self
            .f1
            .take_item()
            .expect("the future has already been polled.");
        let v2 = self
            .f2
            .take_item()
            .expect("the future has already been polled.");
        Ok(Combine::combine(v1, v2).into())
    }
}

// ==== tuples ====

impl<E1, E2> IntoEndpoint for (E1, E2)
where
    E1: IntoEndpoint,
    E2: IntoEndpoint,
    E1::Output: Combine<E2::Output>,
{
    type Output = <E1::Output as Combine<E2::Output>>::Out;
    type Endpoint = And<E1::Endpoint, E2::Endpoint>;

    fn into_endpoint(self) -> Self::Endpoint {
        (And {
            e1: self.0.into_endpoint(),
            e2: self.1.into_endpoint(),
        })
        .with_output::<<E1::Output as Combine<E2::Output>>::Out>()
    }
}

impl<E1, E2, E3> IntoEndpoint for (E1, E2, E3)
where
    E1: IntoEndpoint,
    E2: IntoEndpoint,
    E3: IntoEndpoint,
    E2::Output: Combine<E3::Output>,
    E1::Output: Combine<<E2::Output as Combine<E3::Output>>::Out>,
{
    type Output = <E1::Output as Combine<<E2::Output as Combine<E3::Output>>::Out>>::Out;
    type Endpoint = And<E1::Endpoint, And<E2::Endpoint, E3::Endpoint>>;

    fn into_endpoint(self) -> Self::Endpoint {
        (And {
            e1: self.0.into_endpoint(),
            e2: And {
                e1: self.1.into_endpoint(),
                e2: self.2.into_endpoint(),
            },
        })
        .with_output::<<E1::Output as Combine<<E2::Output as Combine<E3::Output>>::Out>>::Out>()
    }
}

impl<E1, E2, E3, E4> IntoEndpoint for (E1, E2, E3, E4)
where
    E1: IntoEndpoint,
    E2: IntoEndpoint,
    E3: IntoEndpoint,
    E4: IntoEndpoint,
    E3::Output: Combine<E4::Output>,
    E2::Output: Combine<<E3::Output as Combine<E4::Output>>::Out>,
    E1::Output: Combine<<E2::Output as Combine<<E3::Output as Combine<E4::Output>>::Out>>::Out>,
{
    type Output = <E1::Output as Combine<
        <E2::Output as Combine<<E3::Output as Combine<E4::Output>>::Out>>::Out,
    >>::Out;
    type Endpoint = And<E1::Endpoint, And<E2::Endpoint, And<E3::Endpoint, E4::Endpoint>>>;

    fn into_endpoint(self) -> Self::Endpoint {
        (And {
            e1: self.0.into_endpoint(),
            e2: And {
                e1: self.1.into_endpoint(),
                e2: And {
                    e1: self.2.into_endpoint(),
                    e2: self.3.into_endpoint(),
                },
            },
        })
        .with_output::<<E1::Output as Combine<
            <E2::Output as Combine<<E3::Output as Combine<E4::Output>>::Out>>::Out,
        >>::Out>()
    }
}

impl<E1, E2, E3, E4, E5> IntoEndpoint for (E1, E2, E3, E4, E5)
where
    E1: IntoEndpoint,
    E2: IntoEndpoint,
    E3: IntoEndpoint,
    E4: IntoEndpoint,
    E5: IntoEndpoint,
    E4::Output: Combine<E5::Output>,
    E3::Output: Combine<<E4::Output as Combine<E5::Output>>::Out>,
    E2::Output: Combine<<E3::Output as Combine<<E4::Output as Combine<E5::Output>>::Out>>::Out>,
    E1::Output: Combine<
        <E2::Output as Combine<
            <E3::Output as Combine<<E4::Output as Combine<E5::Output>>::Out>>::Out,
        >>::Out,
    >,
{
    type Output = <E1::Output as Combine<
        <E2::Output as Combine<
            <E3::Output as Combine<<E4::Output as Combine<E5::Output>>::Out>>::Out,
        >>::Out,
    >>::Out;
    type Endpoint =
        And<E1::Endpoint, And<E2::Endpoint, And<E3::Endpoint, And<E4::Endpoint, E5::Endpoint>>>>;

    fn into_endpoint(self) -> Self::Endpoint {
        (And {
            e1: self.0.into_endpoint(),
            e2: And {
                e1: self.1.into_endpoint(),
                e2: And {
                    e1: self.2.into_endpoint(),
                    e2: And {
                        e1: self.3.into_endpoint(),
                        e2: self.4.into_endpoint(),
                    },
                },
            },
        })
        .with_output::<<E1::Output as Combine<
            <E2::Output as Combine<
                <E3::Output as Combine<<E4::Output as Combine<E5::Output>>::Out>>::Out,
            >>::Out,
        >>::Out>()
    }
}
