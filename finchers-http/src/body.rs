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

use finchers_core::error::BadRequest;
use finchers_core::input;
use finchers_core::{Bytes, BytesString, Input, Never};
use finchers_endpoint::{Context, Endpoint, Error};
use futures::{Future, Poll};
use std::marker::PhantomData;
use std::str::Utf8Error;
use std::{error, fmt};

/// Creates an endpoint for parsing the incoming request body into the value of `T`
pub fn body<T: FromBody>() -> Body<T> {
    Body { _marker: PhantomData }
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

    fn apply(&self, input: &Input, _: &mut Context) -> Option<Self::Future> {
        match T::is_match(input) {
            true => Some(BodyFuture::Init),
            false => None,
        }
    }
}

#[doc(hidden)]
#[allow(missing_debug_implementations)]
pub enum BodyFuture<T> {
    Init,
    Recv(input::Body),
    Done(PhantomData<fn() -> T>),
}

impl<T: FromBody> Future for BodyFuture<T> {
    type Item = T;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        'poll: loop {
            let next = match *self {
                BodyFuture::Init => {
                    let body = Input::with_mut(|input| input.body()).expect("The body has already taken");
                    BodyFuture::Recv(body.into_data())
                }
                BodyFuture::Recv(ref mut body) => {
                    let buf = try_ready!(body.poll());
                    let body = Input::with_mut(|input| T::from_body(buf, input).map_err(BadRequest::new))?;
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
    type Item = input::BodyStream;
    type Future = BodyStreamFuture;

    fn apply(&self, _: &Input, _: &mut Context) -> Option<Self::Future> {
        Some(BodyStreamFuture { _priv: () })
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct BodyStreamFuture {
    _priv: (),
}

impl Future for BodyStreamFuture {
    type Item = input::BodyStream;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        Input::with_mut(|input| {
            let body = input.body().expect("cannot take a body twice");
            Ok(input::BodyStream::from(body).into())
        })
    }
}

/// The conversion from received request body.
pub trait FromBody: 'static + Sized {
    /// The type of error value returned from `from_body`.
    type Error: error::Error + Send + 'static;

    /// Returns whether the incoming request matches to this type or not.
    ///
    /// This method is used only for the purpose of changing the result of routing.
    /// Otherwise, use `validate` instead.
    #[allow(unused_variables)]
    fn is_match(input: &Input) -> bool {
        true
    }

    /// Performs conversion from raw bytes into itself.
    fn from_body(body: Bytes, input: &mut Input) -> Result<Self, Self::Error>;
}

impl FromBody for () {
    type Error = Never;

    fn from_body(_: Bytes, _: &mut Input) -> Result<Self, Self::Error> {
        Ok(())
    }
}

impl FromBody for Bytes {
    type Error = Never;

    fn from_body(body: Bytes, _: &mut Input) -> Result<Self, Self::Error> {
        Ok(body)
    }
}

impl FromBody for BytesString {
    type Error = Utf8Error;

    fn from_body(body: Bytes, _: &mut Input) -> Result<Self, Self::Error> {
        BytesString::from_shared(body)
    }
}

impl FromBody for String {
    type Error = Utf8Error;

    fn from_body(body: Bytes, _: &mut Input) -> Result<Self, Self::Error> {
        BytesString::from_shared(body).map(Into::into)
    }
}

impl<T: FromBody> FromBody for Option<T> {
    type Error = Never;

    fn from_body(body: Bytes, input: &mut Input) -> Result<Self, Self::Error> {
        Ok(T::from_body(body, input).ok())
    }
}

impl<T: FromBody> FromBody for Result<T, T::Error> {
    type Error = Never;

    fn from_body(body: Bytes, input: &mut Input) -> Result<Self, Self::Error> {
        Ok(T::from_body(body, input))
    }
}
