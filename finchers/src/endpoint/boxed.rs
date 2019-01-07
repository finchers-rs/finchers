use {
    crate::{
        common::Tuple,
        endpoint::{
            ActionContext, //
            Endpoint,
            EndpointAction,
            IsEndpoint,
            Preflight,
            PreflightContext,
        },
        error::Error,
    },
    futures::Poll,
    std::fmt,
};

trait BoxedEndpoint<Bd> {
    type Output: Tuple;

    fn action(&self) -> EndpointActionObj<Bd, Self::Output>;
}

impl<Bd, E> BoxedEndpoint<Bd> for E
where
    E: Endpoint<Bd>,
    E::Action: Send + 'static,
{
    type Output = E::Output;

    fn action(&self) -> EndpointActionObj<Bd, Self::Output> {
        EndpointActionObj {
            inner: Box::new(self.action()),
        }
    }
}

#[allow(missing_docs)]
pub struct EndpointObj<Bd, T>
where
    T: Tuple,
{
    inner: Box<dyn BoxedEndpoint<Bd, Output = T> + Send + Sync + 'static>,
}

impl<Bd, T> EndpointObj<Bd, T>
where
    T: Tuple,
{
    #[allow(missing_docs)]
    pub fn new<E>(endpoint: E) -> Self
    where
        E: Endpoint<Bd, Output = T> + Send + Sync + 'static,
        E::Action: Send + 'static,
    {
        EndpointObj {
            inner: Box::new(endpoint),
        }
    }
}

impl<Bd, T> fmt::Debug for EndpointObj<Bd, T>
where
    T: Tuple,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("EndpointObj").finish()
    }
}

impl<Bd, T> IsEndpoint for EndpointObj<Bd, T> where T: Tuple {}

impl<Bd, T> Endpoint<Bd> for EndpointObj<Bd, T>
where
    T: Tuple,
{
    type Output = T;
    type Action = EndpointActionObj<Bd, T>;

    #[inline]
    fn action(&self) -> Self::Action {
        self.inner.action()
    }
}

#[allow(missing_debug_implementations)]
pub struct EndpointActionObj<Bd, T>
where
    T: Tuple,
{
    inner: Box<dyn EndpointAction<Bd, Output = T> + Send + 'static>,
}

impl<Bd, T> EndpointAction<Bd> for EndpointActionObj<Bd, T>
where
    T: Tuple,
{
    type Output = T;

    #[inline]
    fn preflight(
        &mut self,
        cx: &mut PreflightContext<'_>,
    ) -> Result<Preflight<Self::Output>, Error> {
        self.inner.preflight(cx)
    }

    #[inline]
    fn poll_action(&mut self, cx: &mut ActionContext<'_, Bd>) -> Poll<Self::Output, Error> {
        self.inner.poll_action(cx)
    }
}

// ==== BoxedLocal ====

trait LocalBoxedEndpoint<Bd> {
    type Output: Tuple;

    fn action(&self) -> LocalEndpointActionObj<Bd, Self::Output>;
}

impl<Bd, E> LocalBoxedEndpoint<Bd> for E
where
    E: Endpoint<Bd>,
    E::Action: 'static,
{
    type Output = E::Output;

    fn action(&self) -> LocalEndpointActionObj<Bd, Self::Output> {
        LocalEndpointActionObj {
            inner: Box::new(self.action()),
        }
    }
}

#[allow(missing_docs)]
pub struct LocalEndpointObj<Bd, T>
where
    T: Tuple,
{
    inner: Box<dyn LocalBoxedEndpoint<Bd, Output = T> + 'static>,
}

impl<Bd, T> LocalEndpointObj<Bd, T>
where
    T: Tuple,
{
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

impl<Bd, T> fmt::Debug for LocalEndpointObj<Bd, T>
where
    T: Tuple,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("LocalEndpointObj").finish()
    }
}

impl<Bd, T> IsEndpoint for LocalEndpointObj<Bd, T> where T: Tuple {}

impl<Bd, T> Endpoint<Bd> for LocalEndpointObj<Bd, T>
where
    T: Tuple,
{
    type Output = T;
    type Action = LocalEndpointActionObj<Bd, T>;

    #[inline]
    fn action(&self) -> Self::Action {
        self.inner.action()
    }
}

#[allow(missing_debug_implementations)]
pub struct LocalEndpointActionObj<Bd, T>
where
    T: Tuple,
{
    inner: Box<dyn EndpointAction<Bd, Output = T> + 'static>,
}

impl<Bd, T> EndpointAction<Bd> for LocalEndpointActionObj<Bd, T>
where
    T: Tuple,
{
    type Output = T;

    #[inline]
    fn preflight(
        &mut self,
        cx: &mut PreflightContext<'_>,
    ) -> Result<Preflight<Self::Output>, Error> {
        self.inner.preflight(cx)
    }

    #[inline]
    fn poll_action(&mut self, cx: &mut ActionContext<'_, Bd>) -> Poll<Self::Output, Error> {
        self.inner.poll_action(cx)
    }
}
