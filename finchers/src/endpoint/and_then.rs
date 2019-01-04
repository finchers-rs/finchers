use {
    crate::{
        common::{Func, Tuple},
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
    futures::{Async, Poll},
    std::marker::PhantomData,
};

#[allow(missing_docs)]
#[derive(Debug, Copy, Clone)]
pub struct AndThen<E, F> {
    pub(super) endpoint: E,
    pub(super) f: F,
}

impl<E: IsEndpoint, F> IsEndpoint for AndThen<E, F> {}

impl<E, F, Bd, R> Endpoint<Bd> for AndThen<E, F>
where
    E: Endpoint<Bd>,
    F: Func<E::Output, Out = R> + Clone,
    R: EndpointAction<Bd>,
{
    type Output = (R::Output,);
    type Action = AndThenAction<Bd, E::Action, F::Out, F>;

    fn apply(&self, ecx: &mut ApplyContext<'_, Bd>) -> ApplyResult<Self::Action> {
        let f1 = self.endpoint.apply(ecx)?;
        Ok(AndThenAction {
            try_chain: TryChain::new(f1, self.f.clone()),
        })
    }
}

#[doc(hidden)]
#[allow(missing_debug_implementations)]
pub struct AndThenAction<Bd, F1, F2, F>
where
    F1: EndpointAction<Bd>,
    F2: EndpointAction<Bd>,
    F: Func<F1::Output, Out = F2>,
    F1::Output: Tuple,
{
    try_chain: TryChain<Bd, F1, F2, F>,
}

impl<Bd, F1, F2, F> EndpointAction<Bd> for AndThenAction<Bd, F1, F2, F>
where
    F1: EndpointAction<Bd>,
    F2: EndpointAction<Bd>,
    F: Func<F1::Output, Out = F2>,
    F1::Output: Tuple,
{
    type Output = (F2::Output,);

    fn poll_action(&mut self, cx: &mut ActionContext<'_, Bd>) -> Poll<Self::Output, Error> {
        self.try_chain
            .try_poll(cx, |result, f| match result {
                Ok(ok) => TryChainAction::Action(f.call(ok)),
                Err(err) => TryChainAction::Output(Err(err)),
            })
            .map(|x| x.map(|ok| (ok,)))
    }
}

#[derive(Debug)]
pub enum TryChain<Bd, F1, F2, T>
where
    F1: EndpointAction<Bd>,
    F2: EndpointAction<Bd>,
{
    First(F1, Option<T>),
    Second(F2),
    Empty,
    _Marker(PhantomData<fn(&mut Bd)>),
}

#[allow(missing_debug_implementations)]
pub enum TryChainAction<Bd, F2>
where
    F2: EndpointAction<Bd>,
{
    Action(F2),
    Output(Result<F2::Output, Error>),
}

impl<Bd, F1, F2, T> TryChain<Bd, F1, F2, T>
where
    F1: EndpointAction<Bd>,
    F2: EndpointAction<Bd>,
{
    pub(super) fn new(f1: F1, data: T) -> Self {
        TryChain::First(f1, Some(data))
    }

    pub(super) fn try_poll<F>(
        &mut self,
        cx: &mut ActionContext<'_, Bd>,
        f: F,
    ) -> Poll<F2::Output, Error>
    where
        F: FnOnce(Result<F1::Output, Error>, T) -> TryChainAction<Bd, F2>,
    {
        let mut f = Some(f);
        loop {
            let (out, data) = match self {
                TryChain::First(ref mut f1, ref mut data) => match f1.poll_action(cx) {
                    Ok(Async::NotReady) => return Ok(Async::NotReady),
                    Ok(Async::Ready(ok)) => (Ok(ok), data.take().unwrap()),
                    Err(err) => (Err(err), data.take().unwrap()),
                },
                TryChain::Second(ref mut f2) => return f2.poll_action(cx),
                TryChain::Empty => panic!("This future has already polled."),
                TryChain::_Marker(..) => unreachable!(),
            };

            let f = f.take().unwrap();
            match f(out, data) {
                TryChainAction::Action(f2) => {
                    *self = TryChain::Second(f2);
                    continue;
                }
                TryChainAction::Output(out) => {
                    *self = TryChain::Empty;
                    return out.map(Async::Ready);
                }
            }
        }
    }
}
