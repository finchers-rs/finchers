use super::Wrapper;
use crate::endpoint::{ApplyContext, ApplyResult, Endpoint};

/// Creates a wrapper for creating an endpoint which runs the provided function
/// after calling `Endpoint::apply()`.
pub fn after_apply<F>(f: F) -> AfterApply<F>
where
    F: Fn(&mut ApplyContext<'_>) -> ApplyResult<()>,
{
    AfterApply { f }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct AfterApply<F> {
    f: F,
}

impl<'a, E, F> Wrapper<'a, E> for AfterApply<F>
where
    E: Endpoint<'a>,
    F: Fn(&mut ApplyContext<'_>) -> ApplyResult<()> + 'a,
{
    type Output = E::Output;
    type Endpoint = AfterApplyEndpoint<E, F>;

    fn wrap(self, endpoint: E) -> Self::Endpoint {
        AfterApplyEndpoint {
            endpoint,
            f: self.f,
        }
    }
}

#[allow(missing_docs)]
#[derive(Debug, Copy, Clone)]
pub struct AfterApplyEndpoint<E, F> {
    pub(super) endpoint: E,
    pub(super) f: F,
}

impl<'a, E, F> Endpoint<'a> for AfterApplyEndpoint<E, F>
where
    E: Endpoint<'a>,
    F: Fn(&mut ApplyContext<'_>) -> ApplyResult<()> + 'a,
{
    type Output = E::Output;
    type Future = E::Future;

    #[inline]
    fn apply(&'a self, cx: &mut ApplyContext<'_>) -> ApplyResult<Self::Future> {
        let future = self.endpoint.apply(cx)?;
        (self.f)(cx)?;
        Ok(future)
    }
}
