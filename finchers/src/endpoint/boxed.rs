use {
    crate::{
        common::Tuple,
        endpoint::{
            ActionContext, ApplyContext, ApplyResult, Endpoint, EndpointAction, IsEndpoint,
        },
        error::Error,
    },
    futures::Poll,
    std::fmt,
};

#[allow(missing_debug_implementations)]
pub struct EndpointActionObj<Bd, T: Tuple> {
    inner: Box<dyn EndpointAction<Bd, Output = T> + Send + 'static>,
}

impl<Bd, T: Tuple> EndpointAction<Bd> for EndpointActionObj<Bd, T> {
    type Output = T;

    #[inline]
    fn poll_action(&mut self, cx: &mut ActionContext<'_, Bd>) -> Poll<Self::Output, Error> {
        self.inner.poll_action(cx)
    }
}

trait ActionObjEndpoint<Bd> {
    type Output: Tuple;

    fn apply_obj(
        &self,
        ecx: &mut ApplyContext<'_, Bd>,
    ) -> ApplyResult<EndpointActionObj<Bd, Self::Output>>;
}

impl<Bd, E> ActionObjEndpoint<Bd> for E
where
    E: Endpoint<Bd>,
    E::Action: Send + 'static,
{
    type Output = E::Output;

    #[inline(always)]
    fn apply_obj(
        &self,
        ecx: &mut ApplyContext<'_, Bd>,
    ) -> ApplyResult<EndpointActionObj<Bd, Self::Output>> {
        let future = self.apply(ecx)?;
        Ok(EndpointActionObj {
            inner: Box::new(future),
        })
    }
}

#[allow(missing_docs)]
pub struct EndpointObj<Bd, T: Tuple + 'static> {
    inner: Box<dyn ActionObjEndpoint<Bd, Output = T> + Send + Sync + 'static>,
}

impl<Bd, T: Tuple + 'static> EndpointObj<Bd, T> {
    #[allow(missing_docs)]
    pub fn new<E>(endpoint: E) -> EndpointObj<Bd, T>
    where
        E: Endpoint<Bd, Output = T> + Send + Sync + 'static,
        E::Action: Send + 'static,
    {
        EndpointObj {
            inner: Box::new(endpoint),
        }
    }
}

impl<Bd, T: Tuple + 'static> fmt::Debug for EndpointObj<Bd, T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("EndpointObj").finish()
    }
}

impl<Bd, T: Tuple + 'static> IsEndpoint for EndpointObj<Bd, T> {}

impl<Bd, T: Tuple + 'static> Endpoint<Bd> for EndpointObj<Bd, T> {
    type Output = T;
    type Action = EndpointActionObj<Bd, T>;

    #[inline]
    fn apply(&self, ecx: &mut ApplyContext<'_, Bd>) -> ApplyResult<Self::Action> {
        self.inner.apply_obj(ecx)
    }
}

// ==== BoxedLocal ====
#[allow(missing_debug_implementations)]
pub struct LocalEndpointActionObj<Bd, T: Tuple> {
    inner: Box<dyn EndpointAction<Bd, Output = T> + 'static>,
}

impl<Bd, T: Tuple> EndpointAction<Bd> for LocalEndpointActionObj<Bd, T> {
    type Output = T;

    #[inline]
    fn poll_action(&mut self, cx: &mut ActionContext<'_, Bd>) -> Poll<Self::Output, Error> {
        self.inner.poll_action(cx)
    }
}

trait LocalActionObjEndpoint<Bd> {
    type Output: Tuple;

    fn apply_local_obj(
        &self,
        ecx: &mut ApplyContext<'_, Bd>,
    ) -> ApplyResult<LocalEndpointActionObj<Bd, Self::Output>>;
}

impl<Bd, E: Endpoint<Bd>> LocalActionObjEndpoint<Bd> for E
where
    E::Action: 'static,
{
    type Output = E::Output;

    #[inline(always)]
    fn apply_local_obj(
        &self,
        ecx: &mut ApplyContext<'_, Bd>,
    ) -> ApplyResult<LocalEndpointActionObj<Bd, Self::Output>> {
        let future = self.apply(ecx)?;
        Ok(LocalEndpointActionObj {
            inner: Box::new(future),
        })
    }
}

#[allow(missing_docs)]
pub struct LocalEndpointObj<Bd, T: Tuple + 'static> {
    inner: Box<dyn LocalActionObjEndpoint<Bd, Output = T> + 'static>,
}

impl<Bd, T: Tuple + 'static> LocalEndpointObj<Bd, T> {
    #[allow(missing_docs)]
    pub fn new<E>(endpoint: E) -> Self
    where
        E: Endpoint<Bd, Output = T> + 'static,
        E::Action: 'static,
    {
        LocalEndpointObj {
            inner: Box::new(endpoint),
        }
    }
}

impl<Bd, T: Tuple + 'static> fmt::Debug for LocalEndpointObj<Bd, T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("LocalEndpointObj").finish()
    }
}

impl<Bd, T: Tuple + 'static> IsEndpoint for LocalEndpointObj<Bd, T> {}

impl<Bd, T: Tuple + 'static> Endpoint<Bd> for LocalEndpointObj<Bd, T> {
    type Output = T;
    type Action = LocalEndpointActionObj<Bd, T>;

    #[inline(always)]
    fn apply(&self, ecx: &mut ApplyContext<'_, Bd>) -> ApplyResult<Self::Action> {
        self.inner.apply_local_obj(ecx)
    }
}
