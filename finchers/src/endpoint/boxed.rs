use {
    crate::{
        common::Tuple,
        endpoint::{
            ActionContext, //
            ApplyContext,
            Endpoint,
            EndpointAction,
            IsEndpoint,
            Preflight,
        },
        error::Error,
    },
    futures::Poll,
    std::fmt,
};

trait BoxedEndpoint<Bd> {
    type Output: Tuple;
    type Error: Into<Error>;

    fn action(&self) -> EndpointActionObj<Bd, Self::Output, Self::Error>;
}

impl<Bd, E> BoxedEndpoint<Bd> for E
where
    E: Endpoint<Bd>,
    E::Action: Send + 'static,
{
    type Output = E::Output;
    type Error = E::Error;

    fn action(&self) -> EndpointActionObj<Bd, Self::Output, Self::Error> {
        EndpointActionObj {
            inner: Box::new(self.action()),
        }
    }
}

#[allow(missing_docs)]
pub struct EndpointObj<Bd, T, E>
where
    T: Tuple,
    E: Into<Error>,
{
    inner: Box<dyn BoxedEndpoint<Bd, Output = T, Error = E> + Send + Sync + 'static>,
}

impl<Bd, T, E> EndpointObj<Bd, T, E>
where
    T: Tuple,
    E: Into<Error>,
{
    #[allow(missing_docs)]
    pub fn new(
        endpoint: impl Endpoint<
                Bd,
                Output = T,
                Error = E,
                Action = impl EndpointAction<Bd, Output = T, Error = E> + Send + 'static,
            > + Send
            + Sync
            + 'static,
    ) -> Self {
        EndpointObj {
            inner: Box::new(endpoint),
        }
    }
}

impl<Bd, T, E> fmt::Debug for EndpointObj<Bd, T, E>
where
    T: Tuple,
    E: Into<Error>,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("EndpointObj").finish()
    }
}

impl<Bd, T, E> IsEndpoint for EndpointObj<Bd, T, E>
where
    T: Tuple,
    E: Into<Error>,
{
}

impl<Bd, T, E> Endpoint<Bd> for EndpointObj<Bd, T, E>
where
    T: Tuple,
    E: Into<Error>,
{
    type Output = T;
    type Error = E;
    type Action = EndpointActionObj<Bd, T, E>;

    #[inline]
    fn action(&self) -> Self::Action {
        self.inner.action()
    }
}

#[allow(missing_debug_implementations)]
pub struct EndpointActionObj<Bd, T, E>
where
    T: Tuple,
    E: Into<Error>,
{
    inner: Box<dyn EndpointAction<Bd, Output = T, Error = E> + Send + 'static>,
}

impl<Bd, T, E> EndpointAction<Bd> for EndpointActionObj<Bd, T, E>
where
    T: Tuple,
    E: Into<Error>,
{
    type Output = T;
    type Error = E;

    #[inline]
    fn preflight(
        &mut self,
        cx: &mut ApplyContext<'_>,
    ) -> Result<Preflight<Self::Output>, Self::Error> {
        self.inner.preflight(cx)
    }

    #[inline]
    fn poll_action(&mut self, cx: &mut ActionContext<'_, Bd>) -> Poll<Self::Output, Self::Error> {
        self.inner.poll_action(cx)
    }
}

// ==== BoxedLocal ====

trait LocalBoxedEndpoint<Bd> {
    type Output: Tuple;
    type Error: Into<Error>;

    fn action(&self) -> LocalEndpointActionObj<Bd, Self::Output, Self::Error>;
}

impl<Bd, E> LocalBoxedEndpoint<Bd> for E
where
    E: Endpoint<Bd>,
    E::Action: 'static,
{
    type Output = E::Output;
    type Error = E::Error;

    fn action(&self) -> LocalEndpointActionObj<Bd, Self::Output, Self::Error> {
        LocalEndpointActionObj {
            inner: Box::new(self.action()),
        }
    }
}

#[allow(missing_docs)]
pub struct LocalEndpointObj<Bd, T, E>
where
    T: Tuple,
    E: Into<Error>,
{
    inner: Box<dyn LocalBoxedEndpoint<Bd, Output = T, Error = E> + 'static>,
}

impl<Bd, T, E> LocalEndpointObj<Bd, T, E>
where
    T: Tuple,
    E: Into<Error>,
{
    #[allow(missing_docs)]
    pub fn new(
        endpoint: impl Endpoint<
                Bd,
                Output = T,
                Error = E,
                Action = impl EndpointAction<Bd, Output = T, Error = E> + 'static,
            > + 'static,
    ) -> Self {
        LocalEndpointObj {
            inner: Box::new(endpoint),
        }
    }
}

impl<Bd, T, E> fmt::Debug for LocalEndpointObj<Bd, T, E>
where
    T: Tuple,
    E: Into<Error>,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("LocalEndpointObj").finish()
    }
}

impl<Bd, T, E> IsEndpoint for LocalEndpointObj<Bd, T, E>
where
    T: Tuple,
    E: Into<Error>,
{
}

impl<Bd, T, E> Endpoint<Bd> for LocalEndpointObj<Bd, T, E>
where
    T: Tuple,
    E: Into<Error>,
{
    type Output = T;
    type Error = E;
    type Action = LocalEndpointActionObj<Bd, T, E>;

    #[inline]
    fn action(&self) -> Self::Action {
        self.inner.action()
    }
}

#[allow(missing_debug_implementations)]
pub struct LocalEndpointActionObj<Bd, T, E>
where
    T: Tuple,
    E: Into<Error>,
{
    inner: Box<dyn EndpointAction<Bd, Output = T, Error = E> + 'static>,
}

impl<Bd, T, E> EndpointAction<Bd> for LocalEndpointActionObj<Bd, T, E>
where
    T: Tuple,
    E: Into<Error>,
{
    type Output = T;
    type Error = E;

    #[inline]
    fn preflight(
        &mut self,
        cx: &mut ApplyContext<'_>,
    ) -> Result<Preflight<Self::Output>, Self::Error> {
        self.inner.preflight(cx)
    }

    #[inline]
    fn poll_action(&mut self, cx: &mut ActionContext<'_, Bd>) -> Poll<Self::Output, Self::Error> {
        self.inner.poll_action(cx)
    }
}
