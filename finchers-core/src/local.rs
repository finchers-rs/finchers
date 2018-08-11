//! Utilities for testing endpoints.
//!
//! # Example
//!
//! ```
//! # use finchers_core::endpoints::method::get;
//! # use finchers_core::endpoints::path::{path, param};
//! # use finchers_core::endpoint::EndpointExt;
//! # use finchers_core::generic::one;
//! # use finchers_core::local;
//!
//! let endpoint = get(path("/api/v1/posts").and(param::<u32>()).and(path("stars")))
//!     .map_ok(|id: u32| {
//!         one(format!("id = {}", id))
//!     })
//!     .map_err(drop);
//!
//!
//! let request = local::get("/api/v1/posts/42/stars");
//! let output = request.apply(&endpoint);
//!
//! assert_eq!(output, Some(Ok(("id = 42".into(),))));
//! ```

use std::mem;
use std::mem::PinMut;
use std::task::{Executor, Poll};

use futures_core::future::{Future, TryFuture};
use futures_executor::{self, LocalExecutor, LocalPool};
use futures_util::future::poll_fn;
use http::header::{HeaderName, HeaderValue};
use http::{HttpTryFrom, Method, Request, Uri};

use crate::endpoint::EndpointBase;
use crate::input::{with_set_cx, Cursor, Input, RequestBody};

macro_rules! impl_constructors {
    ($(
        $(#[$doc:meta])*
        $METHOD:ident => $name:ident,
    )*) => {$(
        $(#[$doc])*
        pub fn $name<'a, U>(uri: U) -> LocalRequest<'a>
        where
            Uri: HttpTryFrom<U>,
        {
            LocalRequest::new()
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
pub struct LocalRequest<'a, E: 'a + Executor = LocalExecutor> {
    request: Option<Request<RequestBody>>,
    executor: Option<&'a mut E>,
}

impl LocalRequest<'a> {
    /// Create a new `LocalRequest`.
    pub fn new() -> LocalRequest<'a> {
        LocalRequest {
            request: Some(Request::new(RequestBody::empty())),
            executor: None,
        }
    }

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
    pub fn body(mut self, body: RequestBody) -> Self {
        if let Some(ref mut request) = self.request {
            mem::replace(request.body_mut(), body);
        }
        self
    }

    /// Set the reference to the task executor.
    pub fn executor<E>(self, exec: &'a mut E) -> LocalRequest<'a, E>
    where
        E: Executor,
    {
        LocalRequest {
            request: self.request,
            executor: Some(exec),
        }
    }

    /// Apply this dummy request to the associated endpoint and get its response.
    pub fn apply<E>(self, endpoint: E) -> Option<Result<E::Ok, E::Error>>
    where
        E: EndpointBase,
    {
        let LocalRequest {
            mut request,
            executor,
        } = self;

        let request = request.take().expect("The request has already applied");
        let mut input = Input::new(request);

        let mut in_flight = {
            let input = unsafe { PinMut::new_unchecked(&mut input) };
            let cursor = unsafe { Cursor::new(input.uri().path()) };
            endpoint.apply(input, cursor).map(|res| res.0)
        };

        let future = poll_fn(move |cx| match in_flight {
            Some(ref mut f) => {
                let input = unsafe { PinMut::new_unchecked(&mut input) };
                with_set_cx(input, || {
                    unsafe { PinMut::new_unchecked(f) }.try_poll(cx).map(Some)
                })
            }
            None => Poll::Ready(None),
        });

        block_on(future, executor)
    }
}

fn block_on<F: Future>(future: F, exec: Option<&mut impl Executor>) -> F::Output {
    match exec {
        Some(exec) => {
            let mut pool = LocalPool::new();
            pool.run_until(future, exec)
        }
        None => futures_executor::block_on(future),
    }
}
