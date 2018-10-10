//! The components for using the implementor of `Endpoint` as an HTTP `Service`.

use std::any::TypeId;
use std::error;
use std::fmt;
use std::io;
use std::mem;

use bytes::Buf;
use futures::future;
use futures::{Async, Future, Poll, Stream};
use http::header;
use http::header::HeaderMap;
use http::header::HeaderValue;
use http::{Request, Response};
use hyper::body::{Body, Payload};
use tokio::executor::{Executor, SpawnError};
use tower_service::{NewService, Service};
#[cfg(feature = "tower-web")]
use tower_web::util::buf_stream::{size_hint, BufStream};

use endpoint::context::{ApplyContext, TaskContext};
use endpoint::{with_set_cx, ApplyResult, Cursor, Endpoint, OutputEndpoint};
use error::Error;
use error::Never;
use input::{Input, ReqBody};
use output::body::{Payload as PayloadWrapper, ResBody};
use output::{Output, OutputContext};
use rt::DefaultExecutor;

// ==== App ====

/// A wrapper struct for lifting the instance of `Endpoint` to an HTTP service.
///
/// # Safety
///
/// The implementation of `NewService` for this type internally uses unsafe block
/// with an assumption that `self` always outlives the returned future.
/// Ensure that the all of spawned tasks are terminated and their instance
/// are destroyed before `Self::drop`.
#[derive(Debug)]
pub struct App<E> {
    endpoint: Lift<E>,
}

impl<E> App<E>
where
    for<'a> E: OutputEndpoint<'a> + 'static,
{
    /// Create a new `App` from the specified endpoint.
    pub fn new(endpoint: E) -> App<E> {
        App {
            endpoint: Lift(endpoint),
        }
    }
}

impl<E> NewService for App<E>
where
    for<'a> E: OutputEndpoint<'a> + 'static,
{
    type Request = Request<Body>;
    type Response = Response<AppPayload>;
    type Error = io::Error;
    type Service = AppService<'static, Lift<E>>;
    type InitError = Never;
    type Future = future::FutureResult<Self::Service, Self::InitError>;

    fn new_service(&self) -> Self::Future {
        // This unsafe code assumes that the lifetime of `&self` is always
        // longer than the generated future.
        let endpoint = unsafe { &*(&self.endpoint as *const _) };
        future::ok(AppService { endpoint })
    }
}

#[derive(Debug)]
pub struct Lift<E>(pub(super) E);

impl<'a, E> Endpoint<'a> for Lift<E>
where
    E: OutputEndpoint<'a>,
{
    type Output = E::Output;
    type Future = E::Future;

    #[inline]
    fn apply(&'a self, cx: &mut ApplyContext<'_>) -> ApplyResult<Self::Future> {
        self.0.apply_output(cx)
    }
}

#[derive(Debug)]
pub struct AppService<'e, E: Endpoint<'e>> {
    pub(super) endpoint: &'e E,
}

impl<'e, E> AppService<'e, E>
where
    E: Endpoint<'e>,
{
    pub(crate) fn new(endpoint: &'e E) -> AppService<'e, E> {
        AppService { endpoint }
    }

    pub(crate) fn dispatch(&self, request: Request<ReqBody>) -> AppFuture<'e, E> {
        AppFuture {
            endpoint: self.endpoint,
            state: State::Start(request),
        }
    }
}

impl<'e, E> Service for AppService<'e, E>
where
    E: Endpoint<'e>,
    E::Output: Output,
{
    type Request = Request<Body>;
    type Response = Response<AppPayload>;
    type Error = io::Error;
    type Future = AppFuture<'e, E>;

    fn poll_ready(&mut self) -> Poll<(), Self::Error> {
        Ok(Async::Ready(()))
    }

    fn call(&mut self, request: Self::Request) -> Self::Future {
        self.dispatch(request.map(ReqBody::new))
    }
}

#[derive(Debug)]
pub struct AppFuture<'e, E: Endpoint<'e>> {
    endpoint: &'e E,
    state: State<'e, E>,
}

#[cfg_attr(feature = "cargo-clippy", allow(large_enum_variant))]
enum State<'a, E: Endpoint<'a>> {
    Start(Request<ReqBody>),
    InFlight(Input, E::Future, Cursor),
    Done(Input),
    Gone,
}

impl<'a, E: Endpoint<'a>> fmt::Debug for State<'a, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            State::Start(ref request) => f.debug_struct("Start").field("request", request).finish(),
            State::InFlight(ref input, _, ref cursor) => f
                .debug_struct("InFlight")
                .field("input", input)
                .field("cursor", cursor)
                .finish(),
            State::Done(ref input) => f.debug_struct("Done").field("input", input).finish(),
            State::Gone => f.debug_struct("Gone").finish(),
        }
    }
}

impl<'e, E> AppFuture<'e, E>
where
    E: Endpoint<'e>,
{
    pub(crate) fn poll_apply(&mut self) -> Poll<E::Output, Error> {
        loop {
            let result = match self.state {
                State::Start(..) => None,
                State::InFlight(ref mut input, ref mut f, ref mut cursor) => {
                    let mut tcx = TaskContext::new(input, cursor);
                    match with_set_cx(&mut tcx, || f.poll()) {
                        Ok(Async::NotReady) => return Ok(Async::NotReady),
                        Ok(Async::Ready(ok)) => Some(Ok(ok)),
                        Err(err) => Some(Err(err)),
                    }
                }
                State::Done(..) | State::Gone => panic!("cannot poll AppServiceFuture twice"),
            };

            match (mem::replace(&mut self.state, State::Gone), result) {
                (State::Start(request), None) => {
                    let mut input = Input::new(request);
                    let mut cursor = Cursor::default();
                    match {
                        let mut ecx = ApplyContext::new(&mut input, &mut cursor);
                        self.endpoint.apply(&mut ecx)
                    } {
                        Ok(future) => self.state = State::InFlight(input, future, cursor),
                        Err(err) => {
                            self.state = State::Done(input);
                            return Err(err.into());
                        }
                    }
                }
                (State::InFlight(input, ..), Some(result)) => {
                    self.state = State::Done(input);
                    return result.map(Async::Ready);
                }
                _ => unreachable!("unexpected state"),
            }
        }
    }

    pub(crate) fn poll_all(
        &mut self,
        exec: &mut impl Executor,
    ) -> Poll<Response<AppPayload>, SpawnError>
    where
        E::Output: Output,
    {
        let output = match self.poll_apply() {
            Ok(Async::NotReady) => return Ok(Async::NotReady),
            Ok(Async::Ready(output)) => Ok(output),
            Err(err) => Err(err),
        };

        match mem::replace(&mut self.state, State::Gone) {
            State::Done(mut input) => {
                let output = output.and_then(|output| {
                    output
                        .respond(&mut OutputContext::new(&mut input))
                        .map_err(Into::into)
                });

                let (response, task_opt) = input.finalize(output);
                if let Some(task) = task_opt {
                    exec.spawn(task)?;
                }
                let mut response = response.map(|payload| match payload {
                    Ok(payload) => AppPayload::new(payload),
                    Err(err) => AppPayload::new(PayloadWrapper::from(err.into_payload())),
                });

                // The `content-length` header is automatically insterted by Hyper
                // by using `Payload::content_length()`.
                // However, the instance of `AppPayload` may be convert to another type
                // by middleware and the implementation of `content_length()` does not
                // propagate appropriately.
                // Here is a workaround that presets the value of `content_length()`
                // in advance to prevent the above situation.
                if let Some(len) = response.body().content_length() {
                    response
                        .headers_mut()
                        .entry(header::CONTENT_LENGTH)
                        .expect("should be a valid header name")
                        .or_insert_with(|| {
                            len.to_string()
                                .parse()
                                .expect("should be a valid header value")
                        });
                } else {
                    response
                        .headers_mut()
                        .entry(header::TRANSFER_ENCODING)
                        .expect("should be a valid header name")
                        .or_insert_with(|| HeaderValue::from_static("chunked"));
                }

                response
                    .headers_mut()
                    .entry(header::SERVER)
                    .unwrap()
                    .or_insert_with(|| {
                        HeaderValue::from_static(concat!("finchers/", env!("CARGO_PKG_VERSION")))
                    });
                Ok(Async::Ready(response))
            }
            _ => unreachable!("unexpected condition"),
        }
    }
}

impl<'e, E> Future for AppFuture<'e, E>
where
    E: Endpoint<'e>,
    E::Output: Output,
{
    type Item = Response<AppPayload>;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.poll_all(&mut DefaultExecutor::current()).map_err(|e| {
            error!("failed to spawn an upgraded task: {}", e);
            io::Error::new(io::ErrorKind::Other, e)
        })
    }
}

// ==== AppPayload ====

type BoxedData = Box<dyn Buf + Send + 'static>;
type BoxedError = Box<dyn error::Error + Send + Sync + 'static>;

trait BoxedPayload: Send + 'static {
    fn poll_data_boxed(&mut self) -> Poll<Option<BoxedData>, BoxedError>;
    fn poll_trailers_boxed(&mut self) -> Poll<Option<HeaderMap>, BoxedError>;
    fn is_end_stream_boxed(&self) -> bool;
    fn content_length_boxed(&self) -> Option<u64>;

    // never become a public API.
    #[doc(hidden)]
    fn __private_type_id__(&self) -> TypeId {
        TypeId::of::<Self>()
    }
}

impl<T: Payload> BoxedPayload for T {
    fn poll_data_boxed(&mut self) -> Poll<Option<BoxedData>, BoxedError> {
        self.poll_data()
            .map(|x| x.map(|data_opt| data_opt.map(|data| Box::new(data) as BoxedData)))
            .map_err(Into::into)
    }

    fn poll_trailers_boxed(&mut self) -> Poll<Option<HeaderMap>, BoxedError> {
        self.poll_trailers().map_err(Into::into)
    }

    fn is_end_stream_boxed(&self) -> bool {
        self.is_end_stream()
    }

    fn content_length_boxed(&self) -> Option<u64> {
        self.content_length()
    }
}

/// A payload which will be returned from services generated by `App`.
pub struct AppPayload {
    inner: Box<dyn BoxedPayload>,
}

impl fmt::Debug for AppPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AppPayload").finish()
    }
}

impl AppPayload {
    pub(super) fn new<T>(body: T) -> Self
    where
        T: ResBody,
    {
        AppPayload {
            inner: Box::new(body.into_payload()),
        }
    }

    pub fn is<T: Payload>(&self) -> bool {
        self.inner.__private_type_id__() == TypeId::of::<T>()
    }

    pub fn downcast<T: Payload>(self) -> Result<T, AppPayload> {
        if self.is::<T>() {
            unsafe {
                Ok(*Box::from_raw(
                    Box::into_raw(self.inner) as *mut dyn BoxedPayload as *mut T,
                ))
            }
        } else {
            Err(self)
        }
    }
}

impl Payload for AppPayload {
    type Data = BoxedData;
    type Error = BoxedError;

    #[inline]
    fn poll_data(&mut self) -> Poll<Option<Self::Data>, Self::Error> {
        self.inner.poll_data_boxed()
    }

    #[inline]
    fn poll_trailers(&mut self) -> Poll<Option<HeaderMap>, Self::Error> {
        self.inner.poll_trailers_boxed()
    }

    #[inline]
    fn is_end_stream(&self) -> bool {
        self.inner.is_end_stream_boxed()
    }

    #[inline]
    fn content_length(&self) -> Option<u64> {
        self.inner.content_length_boxed()
    }
}

impl Stream for AppPayload {
    type Item = BoxedData;
    type Error = BoxedError;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        self.poll_data()
    }
}

#[cfg(feature = "tower-web")]
impl BufStream for AppPayload {
    type Item = BoxedData;
    type Error = BoxedError;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        self.poll_data()
    }

    fn size_hint(&self) -> size_hint::SizeHint {
        let mut builder = size_hint::Builder::new();
        if let Some(length) = self.content_length() {
            if length < usize::max_value() as u64 {
                let length = length as usize;
                builder.lower(length).upper(length);
            } else {
                builder.lower(usize::max_value());
            }
        }
        builder.build()
    }
}
