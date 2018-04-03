//! Components for parsing an HTTP request body.
//!
//! The key component is an endpoint `Body<T>`.
//! It will check if the incoming request is valid and start to receive
//! the request body in asynchronous mannar, finally do conversion from
//! received data into the value of `T`.
//!
//! The actual parsing of request body are in implementions of the trait
//! `FromBody`.
//! See [the documentation of `FromBody`][from_body] for details.
//!
//! If you would like to take the *raw* instance of hyper's body stream,
//! use `BodyStream` instead.
//!
//! [from_body]: ../../http/trait.FromBody.html

use futures::{Future, Poll};
use std::marker::PhantomData;
use std::fmt;

use endpoint::{Endpoint, EndpointContext};
use errors::{BadRequest, Error};
use request::{self, with_input, with_input_mut, FromBody, Input};

/// Creates an endpoint for parsing the incoming request body into the value of `T`
pub fn body<T: FromBody>() -> Body<T> {
    Body {
        _marker: PhantomData,
    }
}

#[allow(missing_docs)]
pub struct Body<T> {
    _marker: PhantomData<fn() -> T>,
}

impl<T> Copy for Body<T> {}

impl<T> Clone for Body<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> fmt::Debug for Body<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Body").finish()
    }
}

impl<T: FromBody> Endpoint for Body<T> {
    type Item = T;
    type Future = BodyFuture<T>;

    fn apply(&self, input: &Input, _: &mut EndpointContext) -> Option<Self::Future> {
        match T::is_match(input.parts()) {
            true => Some(BodyFuture::Init),
            false => None,
        }
    }
}

#[doc(hidden)]
#[allow(missing_debug_implementations)]
pub enum BodyFuture<T> {
    Init,
    Recv(request::body::Body),
    Done(PhantomData<fn() -> T>),
}

impl<T: FromBody> Future for BodyFuture<T> {
    type Item = T;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        'poll: loop {
            let next = match *self {
                BodyFuture::Init => {
                    let body = with_input_mut(|input| input.body()).expect("The body has already taken");
                    BodyFuture::Recv(body)
                }
                BodyFuture::Recv(ref mut body) => {
                    let buf = try_ready!(body.poll());
                    let body = with_input(|input| {
                        let request = input.parts();
                        T::from_body(request, &*buf).map_err(BadRequest::new)
                    })?;
                    return Ok(body.into());
                }
                _ => panic!("cannot resolve/reject twice"),
            };
            *self = next;
        }
    }
}

/// Creates an endpoint for taking the instance of `BodyStream`
pub fn body_stream() -> BodyStream {
    BodyStream { _priv: () }
}

#[allow(missing_docs)]
pub struct BodyStream {
    _priv: (),
}

impl Copy for BodyStream {}

impl Clone for BodyStream {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl fmt::Debug for BodyStream {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("BodyStream").finish()
    }
}

impl Endpoint for BodyStream {
    type Item = request::body::BodyStream;
    type Future = BodyStreamFuture;

    fn apply(&self, _: &Input, _: &mut EndpointContext) -> Option<Self::Future> {
        Some(BodyStreamFuture { _priv: () })
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct BodyStreamFuture {
    _priv: (),
}

impl Future for BodyStreamFuture {
    type Item = request::body::BodyStream;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        with_input_mut(|input| {
            let body = input.body_stream().expect("cannot take a body twice");
            Ok(request::body::BodyStream::from(body).into())
        })
    }
}
