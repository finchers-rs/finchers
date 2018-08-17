//! Utilities for testing endpoints.
//!
//! # Example
//!
//! ```
//! # #![feature(rust_2018_preview)]
//! #
//! # use finchers::endpoints::path::{path, param};
//! # use finchers::endpoint::EndpointExt;
//! use finchers::rt::local;
//! use finchers::route;
//!
//! let endpoint = route!(@get / "posts" / u32 / "stars")
//!     .map(|id: u32| format!("id = {}", id));
//!
//! let request = local::get("/posts/42/stars");
//! let output = request.apply(&endpoint);
//!
//! assert_eq!(output, Some(Ok(("id = 42".into(),))));
//! ```

use std::boxed::PinBox;
use std::mem;
use std::mem::PinMut;

use futures_util::compat::TokioDefaultSpawn;
use futures_util::future::poll_fn;
use futures_util::try_future::TryFutureExt;
use http::header::{HeaderName, HeaderValue};
use http::{HttpTryFrom, Method, Request, Uri};
use hyper::body::Body;
use tokio::runtime::current_thread::Runtime;

use crate::app::App;
use crate::endpoint::Endpoint;
use crate::error::Error;
use crate::input::body::ReqBody;

macro_rules! impl_constructors {
    ($(
        $(#[$doc:meta])*
        $METHOD:ident => $name:ident,
    )*) => {$(
        $(#[$doc])*
        pub fn $name<U>(uri: U) -> LocalRequest
        where
            Uri: HttpTryFrom<U>,
        {
            (LocalRequest {
                request: Some(Request::new(ReqBody::from_hyp(Default::default()))),
            })
            .method(Method::$METHOD)
            .uri(uri)
        }
    )*};
}

impl_constructors! {
    /// Create a dummy `GET` request with given URI.
    GET => get,

    /// Create a dummy `POST` request with given URI.
    POST => post,

    /// Create a dummy `PUT` request with given URI.
    PUT => put,

    /// Create a dummy `HEAD` request with given URI.
    HEAD => head,

    /// Create a dummy `DELETE` request with given URI.
    DELETE => delete,

    /// Create a dummy `PATCH` request with given URI.
    PATCH => patch,
}

/// A builder of dummy HTTP request.
#[derive(Debug)]
pub struct LocalRequest {
    request: Option<Request<ReqBody>>,
}

impl LocalRequest {
    /// Overwrite the HTTP method of this dummy request with given value.
    ///
    /// # Panics
    /// This method will panic if the parameter is invalid HTTP method.
    pub fn method<M>(mut self, method: M) -> Self
    where
        Method: HttpTryFrom<M>,
    {
        if let Some(ref mut request) = self.request {
            *request.method_mut() = Method::try_from(method).ok().unwrap();
        }
        self
    }

    /// Overwrite the URI of this dummy request with given value.
    ///
    /// # Panics
    /// This method will panic if the parameter is invalid HTTP method.
    pub fn uri<U>(mut self, uri: U) -> Self
    where
        Uri: HttpTryFrom<U>,
    {
        if let Some(ref mut request) = self.request {
            *request.uri_mut() = Uri::try_from(uri).ok().unwrap();
        }
        self
    }

    /// Append the given header entry into this dummy request.
    ///
    /// # Panics
    /// This method will panic if the given header name or value is invalid.
    pub fn header<K, V>(mut self, name: K, value: V) -> Self
    where
        HeaderName: HttpTryFrom<K>,
        HeaderValue: HttpTryFrom<V>,
    {
        if let Some(ref mut request) = self.request {
            let name = HeaderName::try_from(name).ok().unwrap();
            let value = HeaderValue::try_from(value).ok().unwrap();
            request.headers_mut().insert(name, value);
        }
        self
    }

    /// Overwrite the message body of this dummy request with given instance.
    pub fn body(mut self, body: impl Into<Body>) -> Self {
        if let Some(ref mut request) = self.request {
            mem::replace(request.body_mut(), ReqBody::from_hyp(body.into()));
        }
        self
    }

    /// Apply this dummy request to the associated endpoint and get its response.
    pub fn apply<E>(self, endpoint: E) -> Option<Result<E::Output, Error>>
    where
        E: Endpoint,
    {
        let LocalRequest { mut request } = self;
        let request = request.take().expect("The request has already applied");

        let app = App::new(&endpoint);

        let mut future = app.dispatch_request(request);
        let future = poll_fn(move |cx| {
            let future = unsafe { PinMut::new_unchecked(&mut future) };
            future.poll_output(cx).map(Option::transpose)
        });

        let mut rt = Runtime::new().expect("rt");
        rt.block_on(PinBox::new(future).compat(TokioDefaultSpawn))
            .transpose()
    }
}
