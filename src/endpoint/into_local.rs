use crate::endpoint::{Context, Endpoint, EndpointResult, SendEndpoint};

#[allow(missing_docs)]
#[derive(Debug, Copy, Clone)]
pub struct IntoLocal<E> {
    pub(super) endpoint: E,
}

impl<'a, E: SendEndpoint<'a>> Endpoint<'a> for IntoLocal<E> {
    type Output = E::Output;
    type Future = E::Future;

    #[inline(always)]
    fn apply(&'a self, cx: &mut Context<'_>) -> EndpointResult<Self::Future> {
        self.endpoint.apply(cx)
    }
}
