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

use crate::common::Either;
use crate::endpoint::{Context, Endpoint};
use crate::error::Error;
use crate::input::body::ReqBody;
use crate::input::{with_set_cx, Input};
use crate::output::payload::Once;
use crate::output::Responder;

/// A factory of HTTP service which wraps an `Endpoint`.
#[derive(Debug)]
pub struct App<'e, E: Endpoint<'e>> {
    endpoint: &'e E,
}

impl<'e, E: Endpoint<'e>> App<'e, E> {
    /// Create a new `App` from the provided components.
    pub fn new(endpoint: &'e E) -> App<'e, E> {
        App { endpoint }
    }

    #[allow(missing_docs)]
    pub fn dispatch_request(&self, request: Request<ReqBody>) -> AppFuture<'e, E> {
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
pub struct AppFuture<'e, E: Endpoint<'e>> {
    state: State<E::Future>,
    input: Input,
    endpoint: &'e E,
}

#[derive(Debug)]
enum State<T> {
    Uninitialized,
    InFlight(T),
    Gone,
}

impl<'e, E> AppFuture<'e, E>
where
    E: Endpoint<'e>,
{
    pub fn poll_output(
        self: PinMut<'_, Self>,
        cx: &mut task::Context<'_>,
    ) -> Poll<Result<E::Output, Error>> {
        let this = unsafe { PinMut::get_mut_unchecked(self) };
        let mut input = unsafe { PinMut::new_unchecked(&mut this.input) };

        loop {
            match this.state {
                State::Uninitialized => {
                    let mut ecx = Context::new(input.reborrow());
                    match this.endpoint.apply(&mut ecx) {
                        Ok(future) => this.state = State::InFlight(future),
                        Err(err) => {
                            this.state = State::Gone;
                            return Poll::Ready(Err(err.into()));
                        }
                    }
                }
                State::InFlight(ref mut f) => {
                    let f = unsafe { PinMut::new_unchecked(f) };
                    break with_set_cx(input.reborrow(), || f.try_poll(cx));
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
        let output = ready!(self.reborrow().poll_output(cx));

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

        if let Some(jar) = input.cookie_jar() {
            for cookie in jar.delta() {
                let val = HeaderValue::from_str(&cookie.encoded().to_string()).unwrap();
                response.headers_mut().insert(header::SET_COOKIE, val);
            }
        }

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

impl<'e, E> Future for AppFuture<'e, E>
where
    E: Endpoint<'e>,
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

    impl<'e, E: Endpoint<'e>> NewService for App<'e, E>
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

    impl<'e, E: Endpoint<'e>> Service for App<'e, E>
    where
        E::Output: Responder,
    {
        type ReqBody = Body;
        type ResBody = ResBody<E::Output>;
        type Error = io::Error;
        type Future = Compat<PinBox<AppFuture<'e, E>>, TokioDefaultSpawn>;

        fn call(&mut self, request: Request<Self::ReqBody>) -> Self::Future {
            let future = self.dispatch_request(request.map(ReqBody::from_hyp));
            PinBox::new(future).compat(TokioDefaultSpawn)
        }
    }
}
