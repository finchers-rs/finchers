#![allow(deprecated)]

use crate::endpoint::{Context, Endpoint, EndpointResult};

#[doc(hidden)]
#[deprecated(
    since = "0.12.0-alpha.5",
    note = "use `endpoint::wrappers::before_apply()` instead."
)]
#[derive(Debug, Copy, Clone)]
pub struct BeforeApply<E, F> {
    pub(super) endpoint: E,
    pub(super) f: F,
}

impl<'a, E, F> Endpoint<'a> for BeforeApply<E, F>
where
    E: Endpoint<'a>,
    F: Fn(&mut Context<'_>) -> EndpointResult<()> + 'a,
{
    type Output = E::Output;
    type Future = E::Future;

    #[inline]
    fn apply(&'a self, cx: &mut Context<'_>) -> EndpointResult<Self::Future> {
        (self.f)(cx)?;
        self.endpoint.apply(cx)
    }
}
