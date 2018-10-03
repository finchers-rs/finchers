//! The components for using the implementor of `Endpoint` as an HTTP `Service`.

use std::error;
use std::fmt;
use std::io;

use bytes::Buf;
use either::Either;
use futures::{future, Async, Future, Poll};
use http::header::HeaderMap;
use http::{Request, Response};
use hyper::body::{Body, Payload};
use hyper::service::{NewService, Service};

use common::Tuple;
use endpoint::{with_set_cx, ApplyContext, ApplyResult, Cursor, Endpoint, TaskContext};
use error::Error;
use input::{Input, ReqBody};
use output::body::ResBody;
use output::{Output, OutputContext};

/// A trait which compose the trait bounds representing that
/// the implementor is able to use as an HTTP service.
pub trait IsAppEndpoint: for<'a> AppEndpoint<'a> {}

impl<E> IsAppEndpoint for E where for<'a> E: AppEndpoint<'a> {}

pub trait AppEndpoint<'a>: Send + Sync + 'static {
    type Output: Tuple + Output;
    type Future: Future<Item = Self::Output, Error = Error> + Send + 'a;
    fn apply_app(&'a self, cx: &mut ApplyContext<'_>) -> ApplyResult<Self::Future>;
}

impl<'a, E> AppEndpoint<'a> for E
where
    E: Endpoint<'a> + Send + Sync + 'static,
    E::Output: Output,
    E::Future: Send,
{
    type Output = E::Output;
    type Future = E::Future;

    fn apply_app(&'a self, cx: &mut ApplyContext<'_>) -> ApplyResult<Self::Future> {
        <Self as Endpoint<'a>>::apply(self, cx)
    }
}

#[derive(Debug)]
pub struct Lift<E>(E);

impl<'a, E> Endpoint<'a> for Lift<E>
where
    E: AppEndpoint<'a>,
{
    type Output = E::Output;
    type Future = E::Future;

    #[inline]
    fn apply(&'a self, cx: &mut ApplyContext<'_>) -> ApplyResult<Self::Future> {
        self.0.apply_app(cx)
    }
}

/// A wrapper struct for lifting the instance of `Endpoint` to an HTTP service.
#[derive(Debug)]
pub struct App<E: IsAppEndpoint> {
    endpoint: Lift<E>,
}

impl<E> App<E>
where
    E: IsAppEndpoint,
{
    pub(crate) fn new(endpoint: E) -> App<E> {
        App {
            endpoint: Lift(endpoint),
        }
    }
}

/// # Safety
/// This implementation internally uses some unsafe block with an assumption that
/// `self` always outlives the returned future.
/// Ensure that the all of spawned tasks are terminated and their instance
/// are destroyed before `Self::drop`.
impl<E> NewService for App<E>
where
    E: IsAppEndpoint,
{
    type ReqBody = Body;
    type ResBody = AppPayload;
    type Error = io::Error;
    type Service = AppService<'static, Lift<E>>;
    type InitError = io::Error;
    type Future = future::FutureResult<Self::Service, Self::InitError>;

    fn new_service(&self) -> Self::Future {
        let endpoint = unsafe { &*(&self.endpoint as *const _) };
        future::ok(AppService { endpoint })
    }
}

#[derive(Debug)]
pub struct AppService<'e, E: Endpoint<'e>> {
    endpoint: &'e E,
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
            input: Input::new(request),
            state: State::Uninitialized,
        }
    }
}

impl<'e, E> Service for AppService<'e, E>
where
    E: Endpoint<'e>,
    E::Output: Output,
{
    type ReqBody = Body;
    type ResBody = AppPayload;
    type Error = io::Error;
    type Future = AppFuture<'e, E>;

    fn call(&mut self, request: Request<Self::ReqBody>) -> Self::Future {
        self.dispatch(request.map(ReqBody::from_hyp))
    }
}

#[derive(Debug)]
pub struct AppFuture<'e, E: Endpoint<'e>> {
    state: State<E::Future>,
    input: Input,
    endpoint: &'e E,
}

#[derive(Debug)]
enum State<T> {
    Uninitialized,
    InFlight(T, Cursor),
    Gone,
}

impl<'e, E> AppFuture<'e, E>
where
    E: Endpoint<'e>,
{
    pub(crate) fn poll_endpoint(&mut self) -> Poll<E::Output, Error> {
        loop {
            match self.state {
                State::Uninitialized => {
                    let mut cursor = Cursor::default();
                    match {
                        let mut ecx = ApplyContext::new(&mut self.input, &mut cursor);
                        self.endpoint.apply(&mut ecx)
                    } {
                        Ok(future) => self.state = State::InFlight(future, cursor),
                        Err(err) => {
                            self.state = State::Gone;
                            return Err(err.into());
                        }
                    }
                }
                State::InFlight(ref mut f, ref mut cursor) => {
                    let mut tcx = TaskContext::new(&mut self.input, cursor);
                    break with_set_cx(&mut tcx, || f.poll());
                }
                State::Gone => panic!("cannot poll AppServiceFuture twice"),
            }
        }
    }

    pub(crate) fn poll_output(&mut self) -> Poll<Response<<E::Output as Output>::Body>, Error>
    where
        E::Output: Output,
    {
        let output = try_ready!(self.poll_endpoint());
        let mut cx = OutputContext::new(&mut self.input);
        output
            .respond(&mut cx)
            .map(|res| Async::Ready(res))
            .map_err(Into::into)
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
        let output = match self.poll_output() {
            Ok(Async::Ready(item)) => Ok(item),
            Ok(Async::NotReady) => return Ok(Async::NotReady),
            Err(err) => Err(err),
        };

        Ok(Async::Ready(self.input.finalize_response(output).map(
            |bd| match bd {
                Either::Left(msg) => AppPayload::err(msg),
                Either::Right(bd) => AppPayload::ok(bd),
            },
        )))
    }
}

pub enum AppPayload {
    Err(Option<String>),
    Ok(
        Box<
            dyn Payload<
                Data = Box<dyn Buf + Send + 'static>,
                Error = Box<dyn error::Error + Send + Sync + 'static>,
            >,
        >,
    ),
}

impl fmt::Debug for AppPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppPayload::Err(ref err) => f.debug_tuple("Err").field(err).finish(),
            AppPayload::Ok(..) => f.debug_tuple("Ok").finish(),
        }
    }
}

impl AppPayload {
    fn err(message: String) -> Self {
        AppPayload::Err(Some(message))
    }

    fn ok<T: ResBody>(body: T) -> Self {
        struct Inner<T: Payload>(T);

        impl<T: Payload> Payload for Inner<T> {
            type Data = Box<dyn Buf + Send + 'static>;
            type Error = Box<dyn error::Error + Send + Sync + 'static>;

            #[inline]
            fn poll_data(&mut self) -> Poll<Option<Self::Data>, Self::Error> {
                self.0
                    .poll_data()
                    .map(|x| {
                        x.map(|data_opt| {
                            data_opt.map(|data| Box::new(data) as Box<dyn Buf + Send + 'static>)
                        })
                    }).map_err(Into::into)
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

        AppPayload::Ok(Box::new(Inner(body.into_payload())))
    }
}

impl Payload for AppPayload {
    type Data = Either<io::Cursor<String>, Box<dyn Buf + Send + 'static>>;
    type Error = Box<dyn error::Error + Send + Sync + 'static>;

    #[inline]
    fn poll_data(&mut self) -> Poll<Option<Self::Data>, Self::Error> {
        match self {
            AppPayload::Err(ref mut message) => message
                .take()
                .map(|message| Ok(Async::Ready(Some(Either::Left(io::Cursor::new(message))))))
                .expect("The payload has already polled"),
            AppPayload::Ok(ref mut payload) => payload
                .poll_data()
                .map(|x| x.map(|data_opt| data_opt.map(Either::Right))),
        }
    }

    fn poll_trailers(&mut self) -> Poll<Option<HeaderMap>, Self::Error> {
        match self {
            AppPayload::Err(..) => Ok(Async::Ready(None)),
            AppPayload::Ok(ref mut payload) => payload.poll_trailers(),
        }
    }

    fn is_end_stream(&self) -> bool {
        match self {
            AppPayload::Err(ref msg) => msg.is_none(),
            AppPayload::Ok(ref payload) => payload.is_end_stream(),
        }
    }

    fn content_length(&self) -> Option<u64> {
        match self {
            AppPayload::Err(ref msg) => msg.as_ref().map(|msg| msg.len() as u64),
            AppPayload::Ok(ref payload) => payload.content_length(),
        }
    }
}
