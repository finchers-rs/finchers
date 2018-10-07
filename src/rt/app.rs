//! The components for using the implementor of `Endpoint` as an HTTP `Service`.

use futures::future;
use http::{Request, Response};
use hyper::body::Body;
use std::io;
use tower_service::NewService;

pub use self::app_payload::AppPayload;
use self::app_service::{AppService, Lift};

use endpoint::{Endpoint, OutputEndpoint};
use error::Never;
use output::Output;

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

/// A wrapper struct for lifting a reference to `Endpoint` to an HTTP service.
#[derive(Debug)]
pub struct AppRef<'a, E: 'a> {
    endpoint: &'a E,
}

impl<'a, E> AppRef<'a, E>
where
    E: Endpoint<'a>,
    E::Output: Output,
{
    /// Create a new `AppRef` from a reference to the specified endpoint.
    pub fn new(endpoint: &'a E) -> AppRef<'a, E> {
        AppRef { endpoint }
    }
}

impl<'a, E> NewService for AppRef<'a, E>
where
    E: Endpoint<'a>,
    E::Output: Output,
{
    type Request = Request<Body>;
    type Response = Response<AppPayload>;
    type Error = io::Error;
    type Service = AppService<'a, E>;
    type InitError = Never;
    type Future = future::FutureResult<Self::Service, Self::InitError>;

    fn new_service(&self) -> Self::Future {
        future::ok(AppService {
            endpoint: self.endpoint,
        })
    }
}

pub(crate) mod app_service {
    use std::fmt;
    use std::io;
    use std::mem;

    use futures::{Async, Future, Poll};
    use http::header;
    use http::header::HeaderValue;
    use http::{Request, Response};
    use hyper::body::{Body, Payload as _Payload};
    use tokio::executor::{Executor, SpawnError};
    use tower_service::Service;

    use endpoint::context::{ApplyContext, TaskContext};
    use endpoint::{with_set_cx, ApplyResult, Cursor, Endpoint, OutputEndpoint};
    use error::Error;
    use input::{Input, ReqBody};
    use output::{Output, OutputContext};
    use rt::DefaultExecutor;

    use super::AppPayload;

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

    enum State<'a, E: Endpoint<'a>> {
        Start(Request<ReqBody>),
        InFlight(Input, E::Future, Cursor),
        Done(Input),
        Gone,
    }

    impl<'a, E: Endpoint<'a>> fmt::Debug for State<'a, E> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                State::Start(ref request) => {
                    f.debug_struct("Start").field("request", request).finish()
                }
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
        pub(crate) fn poll_endpoint(&mut self) -> Poll<E::Output, Error> {
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

        pub(crate) fn poll_output(&mut self) -> Poll<Response<<E::Output as Output>::Body>, Error>
        where
            E::Output: Output,
        {
            let output = try_ready!(self.poll_endpoint());
            match self.state {
                State::Done(ref mut input) => {
                    let mut cx = OutputContext::new(input);
                    output
                        .respond(&mut cx)
                        .map(|res| Async::Ready(res))
                        .map_err(Into::into)
                }
                _ => unreachable!("unexpected condition"),
            }
        }

        pub(crate) fn poll_all(
            &mut self,
            exec: &mut impl Executor,
        ) -> Poll<Response<AppPayload>, SpawnError>
        where
            E::Output: Output,
        {
            let output = match self.poll_output() {
                Ok(Async::Ready(item)) => Ok(item),
                Ok(Async::NotReady) => return Ok(Async::NotReady),
                Err(err) => Err(err),
            };

            match mem::replace(&mut self.state, State::Gone) {
                State::Done(input) => {
                    let (response, task_opt) = input.finalize(output);
                    if let Some(task) = task_opt {
                        exec.spawn(task)?;
                    }
                    let mut response = response.map(AppPayload::new);

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
                            HeaderValue::from_static(concat!(
                                "finchers/",
                                env!("CARGO_PKG_VERSION")
                            ))
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
}

mod app_payload {
    use std::error;
    use std::fmt;
    use std::io;

    use bytes::Buf;
    use either::Either;
    use futures::{Async, Poll, Stream};
    use http::header::HeaderMap;
    use hyper::body::Payload;

    use output::body::ResBody;

    type AppPayloadData = Either<io::Cursor<String>, Box<dyn Buf + Send + 'static>>;
    type BoxedData = Box<dyn Buf + Send + 'static>;
    type BoxedError = Box<dyn error::Error + Send + Sync + 'static>;
    type BoxedPayload = Box<dyn Payload<Data = BoxedData, Error = BoxedError>>;

    /// A payload which will be returned from services generated by `App`.
    pub struct AppPayload {
        inner: Either<Option<String>, BoxedPayload>,
    }

    impl fmt::Debug for AppPayload {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self.inner {
                Either::Left(ref err) => f.debug_tuple("Err").field(err).finish(),
                Either::Right(..) => f.debug_tuple("Ok").finish(),
            }
        }
    }

    impl AppPayload {
        pub(super) fn new<T>(body: Either<String, T>) -> Self
        where
            T: ResBody,
        {
            match body {
                Either::Left(message) => Self::err(message),
                Either::Right(body) => Self::ok(body),
            }
        }

        fn err(message: String) -> Self {
            AppPayload {
                inner: Either::Left(Some(message)),
            }
        }

        fn ok<T: ResBody>(body: T) -> Self {
            struct Inner<T: Payload>(T);

            impl<T: Payload> Payload for Inner<T> {
                type Data = BoxedData;
                type Error = BoxedError;

                #[inline]
                fn poll_data(&mut self) -> Poll<Option<Self::Data>, Self::Error> {
                    self.0
                        .poll_data()
                        .map(|x| x.map(|data_opt| data_opt.map(|data| Box::new(data) as BoxedData)))
                        .map_err(Into::into)
                }

                fn poll_trailers(&mut self) -> Poll<Option<HeaderMap>, Self::Error> {
                    self.0.poll_trailers().map_err(Into::into)
                }

                fn is_end_stream(&self) -> bool {
                    self.0.is_end_stream()
                }

                fn content_length(&self) -> Option<u64> {
                    self.0.content_length()
                }
            }

            AppPayload {
                inner: Either::Right(Box::new(Inner(body.into_payload()))),
            }
        }
    }

    impl Payload for AppPayload {
        type Data = AppPayloadData;
        type Error = BoxedError;

        #[inline]
        fn poll_data(&mut self) -> Poll<Option<Self::Data>, Self::Error> {
            match self.inner {
                Either::Left(ref mut message) => message
                    .take()
                    .map(|message| Ok(Async::Ready(Some(Either::Left(io::Cursor::new(message))))))
                    .expect("The payload has already polled"),
                Either::Right(ref mut payload) => payload
                    .poll_data()
                    .map(|x| x.map(|data_opt| data_opt.map(Either::Right))),
            }
        }

        fn poll_trailers(&mut self) -> Poll<Option<HeaderMap>, Self::Error> {
            match self.inner {
                Either::Left(..) => Ok(Async::Ready(None)),
                Either::Right(ref mut payload) => payload.poll_trailers(),
            }
        }

        fn is_end_stream(&self) -> bool {
            match self.inner {
                Either::Left(ref msg) => msg.is_none(),
                Either::Right(ref payload) => payload.is_end_stream(),
            }
        }

        fn content_length(&self) -> Option<u64> {
            match self.inner {
                Either::Left(ref msg) => msg.as_ref().map(|msg| msg.len() as u64),
                Either::Right(ref payload) => payload.content_length(),
            }
        }
    }

    impl Stream for AppPayload {
        type Item = AppPayloadData;
        type Error = BoxedError;

        fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
            self.poll_data()
        }
    }

    #[cfg(feature = "tower-web")]
    mod imp_buf_stream_for_app_payload {
        use super::*;

        use futures::Poll;
        use hyper::body::Payload;
        use tower_web::util::buf_stream::size_hint;
        use tower_web::util::BufStream;

        impl BufStream for AppPayload {
            type Item = AppPayloadData;
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
    }
}
