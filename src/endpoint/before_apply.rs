use crate::endpoint::{Context, Endpoint, EndpointResult};

#[allow(missing_docs)]
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
