//! The components for using the implementor of `Endpoint` as an HTTP `Service`.

use std::any::TypeId;
use std::error;
use std::fmt;
use std::io;
use std::marker::PhantomData;
use std::mem;
use std::sync::Arc;

use bytes::Buf;
use futures::future;
use futures::{Async, Future, Poll, Stream};
use http::header;
use http::header::HeaderMap;
use http::header::HeaderValue;
use http::{Request, Response};
use hyper::body::Payload;
use tower_service::{NewService, Service};

use crate::endpoint::context::ApplyContext;
use crate::endpoint::{Cursor, Endpoint};
use crate::error::Error;
use crate::error::Never;
use crate::future::{Context, EndpointFuture};
use crate::input::Input;
use crate::output::body::{Payload as PayloadWrapper, ResBody};
use crate::output::IntoResponse;

pub trait EndpointServiceExt<Bd>: Endpoint<Bd> + Sized {
    fn into_service(self) -> App<Bd, Self>;
}

impl<E, Bd> EndpointServiceExt<Bd> for E
where
    E: Endpoint<Bd>,
{
    fn into_service(self) -> App<Bd, Self> {
        App::new(self)
    }
}

/// A wrapper struct for lifting the instance of `Endpoint` to an HTTP service.
#[derive(Debug)]
pub struct App<Bd, E: Endpoint<Bd>> {
    endpoint: Arc<E>,
    _marker: PhantomData<fn(Bd)>,
}

impl<Bd, E> App<Bd, E>
where
    E: Endpoint<Bd>,
{
    /// Create a new `App` from the specified endpoint.
    pub fn new(endpoint: E) -> Self {
        App {
            endpoint: Arc::new(endpoint),
            _marker: PhantomData,
        }
    }
}

impl<Bd, E> NewService for App<Bd, E>
where
    E: Endpoint<Bd>,
    E::Output: IntoResponse,
    <E::Output as IntoResponse>::Body: ResBody,
{
    type Request = Request<Bd>;
    type Response = Response<AppPayload>;
    type Error = io::Error;
    type Service = AppService<Bd, Arc<E>>;
    type InitError = Never;
    type Future = future::FutureResult<Self::Service, Self::InitError>;

    fn new_service(&self) -> Self::Future {
        future::ok(AppService::new(self.endpoint.clone()))
    }
}

#[derive(Debug)]
pub struct AppService<Bd, E: Endpoint<Bd>> {
    pub(super) endpoint: E,
    _marker: PhantomData<fn(Bd)>,
}

impl<Bd, E> AppService<Bd, E>
where
    E: Endpoint<Bd> + Clone,
{
    pub(crate) fn new(endpoint: E) -> Self {
        AppService {
            endpoint,
            _marker: PhantomData,
        }
    }

    pub(crate) fn dispatch(&self, request: Request<Bd>) -> AppFuture<Bd, E> {
        AppFuture {
            endpoint: self.endpoint.clone(),
            state: State::Start(request),
        }
    }
}

impl<Bd, E> Service for AppService<Bd, E>
where
    E: Endpoint<Bd> + Clone,
    E::Output: IntoResponse,
    <E::Output as IntoResponse>::Body: ResBody,
{
    type Request = Request<Bd>;
    type Response = Response<AppPayload>;
    type Error = io::Error;
    type Future = AppFuture<Bd, E>;

    fn poll_ready(&mut self) -> Poll<(), Self::Error> {
        Ok(Async::Ready(()))
    }

    fn call(&mut self, request: Self::Request) -> Self::Future {
        self.dispatch(request)
    }
}

#[derive(Debug)]
pub struct AppFuture<Bd, E: Endpoint<Bd>> {
    endpoint: E,
    state: State<Bd, E>,
}

#[allow(clippy::large_enum_variant)]
enum State<Bd, E: Endpoint<Bd>> {
    Start(Request<Bd>),
    InFlight(Input<Bd>, E::Future, Cursor),
    Done(Input<Bd>),
    Gone,
}

impl<Bd, E> fmt::Debug for State<Bd, E>
where
    Bd: fmt::Debug,
    E: Endpoint<Bd>,
{
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

impl<Bd, E> AppFuture<Bd, E>
where
    E: Endpoint<Bd>,
{
    pub(crate) fn poll_apply(&mut self) -> Poll<E::Output, Error> {
        loop {
            let result = match self.state {
                State::Start(..) => None,
                State::InFlight(ref mut input, ref mut f, ref mut cursor) => {
                    let mut tcx = Context::new(input, cursor);
                    match f.poll_endpoint(&mut tcx) {
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

    pub(crate) fn poll_all(&mut self) -> Poll<Response<AppPayload>, io::Error>
    where
        E::Output: IntoResponse,
        <E::Output as IntoResponse>::Body: ResBody,
    {
        let output = match self.poll_apply() {
            Ok(Async::NotReady) => return Ok(Async::NotReady),
            Ok(Async::Ready(output)) => Ok(output),
            Err(err) => Err(err),
        };

        match mem::replace(&mut self.state, State::Gone) {
            State::Done(input) => {
                let output = output.map(|output| output.into_response(input.request()));

                let response = input.finalize(output);
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

impl<Bd, E> Future for AppFuture<Bd, E>
where
    E: Endpoint<Bd>,
    E::Output: IntoResponse,
    <E::Output as IntoResponse>::Body: ResBody,
{
    type Item = Response<AppPayload>;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.poll_all()
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
