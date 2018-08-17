//! The components to construct an asynchronous HTTP service from the `Endpoint`.

#![allow(missing_docs)]

use std::future::Future;
use std::io;
use std::mem::PinMut;
use std::task;
use std::task::Poll;

use futures_core::future::TryFuture;
use futures_util::ready;
use http::header::HeaderValue;
use http::{header, Request, Response};

use crate::endpoint::Endpoint;
use crate::error::{Error, NoRoute};
use crate::generic::Either;
use crate::input::body::ReqBody;
use crate::input::{with_set_cx, Cursor, Input};
use crate::output::payload::Once;
use crate::output::Responder;

/// A factory of HTTP service which wraps an `Endpoint`.
#[derive(Debug)]
pub struct App<'a, E: Endpoint + 'a> {
    endpoint: &'a E,
}

impl<'a, E> App<'a, E>
where
    E: Endpoint + 'a,
{
    /// Create a new `App` from the provided components.
    pub fn new(endpoint: &'a E) -> App<'a, E> {
        App { endpoint }
    }

    #[allow(missing_docs)]
    pub fn dispatch_request(&self, request: Request<ReqBody>) -> AppFuture<'a, E> {
        AppFuture {
            state: State::Uninitialized,
            input: Input::new(request),
            endpoint: self.endpoint,
        }
    }
}

#[allow(type_alias_bounds)]
pub type ResBody<T: Responder> = Either<Once<String>, T::Body>;

#[allow(missing_docs)]
#[derive(Debug)]
pub struct AppFuture<'a, E: Endpoint + 'a> {
    state: State<E::Future>,
    input: Input,
    endpoint: &'a E,
}

#[derive(Debug)]
enum State<T> {
    Uninitialized,
    InFlight(T),
    Gone,
}

impl<'a, E> AppFuture<'a, E>
where
    E: Endpoint + 'a,
{
    pub fn poll_output(
        self: PinMut<'_, Self>,
        cx: &mut task::Context<'_>,
    ) -> Poll<Option<Result<E::Output, Error>>> {
        let this = unsafe { PinMut::get_mut_unchecked(self) };
        let mut input = unsafe { PinMut::new_unchecked(&mut this.input) };

        loop {
            match this.state {
                State::Uninitialized => {
                    let result = {
                        let cursor = unsafe { Cursor::new(&*(input.uri().path() as *const str)) };
                        this.endpoint.apply(input.reborrow(), cursor)
                    };
                    match result {
                        Some((future, _cursor)) => this.state = State::InFlight(future),
                        None => {
                            this.state = State::Gone;
                            return Poll::Ready(None);
                        }
                    }
                }
                State::InFlight(ref mut f) => {
                    let f = unsafe { PinMut::new_unchecked(f) };
                    break with_set_cx(input.reborrow(), || f.try_poll(cx)).map(Some);
                }
                State::Gone => panic!("cannot poll AppServiceFuture twice"),
            }
        }
    }

    pub fn poll_response(
        mut self: PinMut<'_, Self>,
        cx: &mut task::Context<'_>,
    ) -> Poll<Response<ResBody<E::Output>>>
    where
        E::Output: Responder,
    {
        let output = ready!(self.reborrow().poll_output(cx)).unwrap_or_else(|| Err(NoRoute.into()));

        let this = unsafe { PinMut::get_mut_unchecked(self) };
        let mut input = unsafe { PinMut::new_unchecked(&mut this.input) };
        let mut response = output
            .and_then({
                let input = input.reborrow();
                |out| {
                    out.respond(input)
                        .map(|res| res.map(Either::Right))
                        .map_err(Into::into)
                }
            }).unwrap_or_else(|err| err.to_response().map(|body| Either::Left(Once::new(body))));

        response
            .headers_mut()
            .entry(header::SERVER)
            .unwrap()
            .or_insert(HeaderValue::from_static(concat!(
                "finchers-runtime/",
                env!("CARGO_PKG_VERSION")
            )));

        Poll::Ready(response)
    }
}

impl<'a, E> Future for AppFuture<'a, E>
where
    E: Endpoint + 'a,
    E::Output: Responder,
{
    type Output = io::Result<Response<ResBody<E::Output>>>;

    fn poll(self: PinMut<'_, Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        self.poll_response(cx).map(Ok)
    }
}

mod service {
    use super::{App, AppFuture, ResBody};

    use std::boxed::PinBox;
    use std::io;

    use futures as futures01;
    use futures_util::compat::{Compat, TokioDefaultSpawn};
    use futures_util::try_future::TryFutureExt;
    use http::Request;
    use hyper::body::Body;
    use hyper::service::{NewService, Service};

    use crate::endpoint::Endpoint;
    use crate::input::body::ReqBody;
    use crate::output::Responder;

    impl<'a, E: Endpoint> NewService for App<'a, E>
    where
        E::Output: Responder,
    {
        type ReqBody = Body;
        type ResBody = ResBody<E::Output>;
        type Error = io::Error;
        type Service = Self;
        type InitError = io::Error;
        type Future = futures01::future::FutureResult<Self::Service, Self::InitError>;

        fn new_service(&self) -> Self::Future {
            futures01::future::ok(App {
                endpoint: self.endpoint,
            })
        }
    }

    impl<'a, E: Endpoint> Service for App<'a, E>
    where
        E::Output: Responder,
    {
        type ReqBody = Body;
        type ResBody = ResBody<E::Output>;
        type Error = io::Error;
        type Future = Compat<PinBox<AppFuture<'a, E>>, TokioDefaultSpawn>;

        fn call(&mut self, request: Request<Self::ReqBody>) -> Self::Future {
            let future = self.dispatch_request(request.map(ReqBody::from_hyp));
            PinBox::new(future).compat(TokioDefaultSpawn)
        }
    }
}
