//! Components for managing HTTP server.

use failure::{err_msg, Fallible};
use futures::future::poll_fn;
use futures::Future;
use hyper::server::conn::Http;
use hyper::server::Builder;
use std::net::ToSocketAddrs;
use tokio::net::TcpListener;
use tokio::runtime::Runtime;

use app::App;
use endpoint::Endpoint;
use output::Output;

#[cfg(feature = "rt")]
use rt::blocking::{with_set_runtime_mode, RuntimeMode};

#[cfg(feature = "rt")]
fn annotate<R>(f: impl FnOnce() -> R) -> R {
    with_set_runtime_mode(RuntimeMode::ThreadPool, f)
}

#[cfg(not(feature = "rt"))]
fn annotate<R>(f: impl FnOnce() -> R) -> R {
    f()
}

// ==== LaunchEndpoint ====

/// A trait representing a constraint used in the definition of `Launcher<E>`.
pub trait LaunchEndpoint<'a>: sealed::Sealed<'a> {}

impl<'a, E> LaunchEndpoint<'a> for E
where
    E: Endpoint<'a> + Send + Sync + 'static,
    E::Output: Output,
    E::Future: Send,
{}

mod sealed {
    use futures::Future;

    use common::Tuple;
    use endpoint::{ApplyContext, ApplyResult, Endpoint};
    use error::Error;
    use output::Output;

    pub trait Sealed<'a>: Send + Sync + 'static {
        type Output: Tuple + Output;
        type Future: Future<Item = Self::Output, Error = Error> + Send + 'a;

        fn apply(&'a self, cx: &mut ApplyContext<'_>) -> ApplyResult<Self::Future>;

        fn into_endpoint(self) -> IntoEndpoint<Self>
        where
            Self: Sized,
        {
            IntoEndpoint(self)
        }
    }

    impl<'a, E> Sealed<'a> for E
    where
        E: Endpoint<'a> + Send + Sync + 'static,
        E::Output: Output,
        E::Future: Send,
    {
        type Output = E::Output;
        type Future = E::Future;

        fn apply(&'a self, cx: &mut ApplyContext<'_>) -> ApplyResult<Self::Future> {
            <Self as Endpoint<'a>>::apply(self, cx)
        }
    }

    #[derive(Debug)]
    pub struct IntoEndpoint<E>(E);

    impl<'e, E: Sealed<'e>> Endpoint<'e> for IntoEndpoint<E> {
        type Output = E::Output;
        type Future = E::Future;

        fn apply(&'e self, cx: &mut ApplyContext<'_>) -> ApplyResult<Self::Future> {
            self.0.apply(cx)
        }
    }
}

// ==== Launcher ====

/// A launcher of HTTP server which contains an endpoint and some configurations.
#[derive(Debug)]
pub struct Launcher<E>
where
    for<'e> E: LaunchEndpoint<'e>,
{
    endpoint: E,
    http: Option<Http>,
    rt: Option<Runtime>,
}

impl<E> Launcher<E>
where
    for<'e> E: LaunchEndpoint<'e>,
{
    /// Sets the protocol-level configuration.
    pub fn http(self, http: Http) -> Self {
        Launcher {
            http: Some(http),
            ..self
        }
    }

    /// Sets the instance of configured Tokio runtime.
    pub fn runtime(self, rt: Runtime) -> Self {
        Launcher {
            rt: Some(rt),
            ..self
        }
    }

    /// Start the server with binding the specified listener address.
    pub fn start(self, addr: impl ToSocketAddrs) {
        if let Err(err) = self.start_inner(addr) {
            error!("launch error: {}", err);
        }
    }

    fn start_inner(self, addr: impl ToSocketAddrs) -> Fallible<()> {
        let Launcher { endpoint, rt, http } = self;

        let addr = addr
            .to_socket_addrs()?
            .next()
            .ok_or_else(|| err_msg("empty listener address"))?;
        let incoming = TcpListener::bind(&addr)?.incoming();

        // Acquire a `'static` reference to the target endpoint.
        //
        // This is an unsafe operation necessary to execute the following future
        // with Tokio runtime.
        let endpoint = endpoint.into_endpoint();
        let endpoint: &'static _ = unsafe { &*(&endpoint as *const _) };
        let new_service = App::new(endpoint);

        let http = http.unwrap_or_else(Http::new);
        let mut server = Builder::new(incoming, http)
            .serve(new_service)
            .map_err(|err| error!("server error: {}", err));
        let server = poll_fn(move || annotate(|| server.poll()));

        let mut rt = match rt {
            Some(rt) => rt,
            None => Runtime::new()?,
        };
        rt.spawn(server);
        rt.shutdown_on_idle().wait().unwrap();

        Ok(())
    }
}

/// Create an instance of `Launcher` from the specified endpoint.
///
/// # Example
///
/// ```ignore
/// fn main() -> Fallible<()> {
///     let endpoint = ...;
///
///     info!("Listening on http://{}", addr);
///     launch(endpoint)
///         .start(([127, 0, 0, 1], 5000))
/// }
/// ```
pub fn launch<E>(endpoint: E) -> Launcher<E>
where
    for<'e> E: LaunchEndpoint<'e>,
{
    Launcher {
        endpoint,
        http: None,
        rt: None,
    }
}
