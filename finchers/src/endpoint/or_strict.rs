use {
    crate::{
        endpoint::{
            ActionContext, //
            ApplyContext,
            ApplyResult,
            Endpoint,
            EndpointAction,
            IsEndpoint,
        },
        error::Error,
    },
    either::Either,
    futures::Poll,
};

#[allow(missing_docs)]
#[derive(Debug, Copy, Clone)]
pub struct OrStrict<E1, E2> {
    pub(super) e1: E1,
    pub(super) e2: E2,
}

impl<E1: IsEndpoint, E2: IsEndpoint> IsEndpoint for OrStrict<E1, E2> {}

impl<E1, E2, Bd> Endpoint<Bd> for OrStrict<E1, E2>
where
    E1: Endpoint<Bd>,
    E2: Endpoint<Bd, Output = E1::Output>,
{
    type Output = E1::Output;
    type Action = OrStrictAction<E1::Action, E2::Action>;

    fn apply(&self, ecx: &mut ApplyContext<'_, Bd>) -> ApplyResult<Self::Action> {
        let orig_cursor = ecx.cursor().clone();
        match self.e1.apply(ecx) {
            Ok(future1) => {
                *ecx.cursor() = orig_cursor;
                Ok(OrStrictAction::left(future1))
            }
            Err(err1) => match self.e2.apply(ecx) {
                Ok(future) => Ok(OrStrictAction::right(future)),
                Err(err2) => Err(err1.merge(err2)),
            },
        }
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct OrStrictAction<L, R> {
    inner: Either<L, R>,
}

impl<L, R> OrStrictAction<L, R> {
    fn left(l: L) -> Self {
        OrStrictAction {
            inner: Either::Left(l),
        }
    }

    fn right(r: R) -> Self {
        OrStrictAction {
            inner: Either::Right(r),
        }
    }
}

impl<L, R, Bd> EndpointAction<Bd> for OrStrictAction<L, R>
where
    L: EndpointAction<Bd>,
    R: EndpointAction<Bd, Output = L::Output>,
{
    type Output = L::Output;

    #[inline]
    fn poll_action(&mut self, cx: &mut ActionContext<'_, Bd>) -> Poll<Self::Output, Error> {
        match self.inner {
            Either::Left(ref mut t) => t.poll_action(cx),
            Either::Right(ref mut t) => t.poll_action(cx),
        }
    }
}
