//! A testing framework for Finchers.

#![doc(html_root_url = "https://docs.rs/finchers-test/0.11.0")]
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![deny(warnings)]

extern crate finchers_core;
extern crate futures;
extern crate http;

use futures::{Async, Future};
use http::header::{HeaderName, HeaderValue};
use http::{HttpTryFrom, Method, Request, Uri};
use std::mem;

use finchers_core::endpoint::ApplyRequest;
use finchers_core::input::RequestBody;
use finchers_core::{Endpoint, Error, Input, Never, Poll, Task};

/// A wrapper struct of an endpoint which adds the facility for testing.
#[derive(Debug)]
pub struct Client<E: Endpoint> {
    endpoint: E,
}

impl<E: Endpoint> Client<E> {
    /// Create a new instance of `Client` from a given endpoint.
    pub fn new(endpoint: E) -> Client<E> {
        Client { endpoint }
    }

    /// Create a dummy request with given HTTP method and URI.
    pub fn request<'a, M, U>(&'a self, method: M, uri: U) -> ClientRequest<'a, E>
    where
        Method: HttpTryFrom<M>,
        Uri: HttpTryFrom<U>,
    {
        let mut client = ClientRequest {
            client: self,
            request: Request::new(()),
            body: None,
        };
        client.method(method);
        client.uri(uri);
        client
    }
}

macro_rules! impl_constructors {
    ($(
        $(#[$doc:meta])*
        $METHOD:ident => $name:ident,
    )*) => {$(
        $(#[$doc])*
        pub fn $name<'a, U>(&'a self, uri: U) -> ClientRequest<'a, E>
        where
            Uri: HttpTryFrom<U>,
        {
            self.request(Method::$METHOD, uri)
        }
    )*};
}

impl<E: Endpoint> Client<E> {
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
}

/// A builder of dummy HTTP request.
#[derive(Debug)]
pub struct ClientRequest<'a, E: Endpoint + 'a> {
    client: &'a Client<E>,
    request: Request<()>,
    body: Option<RequestBody>,
}

impl<'a, E: Endpoint> ClientRequest<'a, E> {
    /// Overwrite the HTTP method of this dummy request with given value.
    ///
    /// # Panics
    /// This method will panic if the parameter is invalid HTTP method.
    pub fn method<M>(&mut self, method: M) -> &mut ClientRequest<'a, E>
    where
        Method: HttpTryFrom<M>,
    {
        *self.request.method_mut() = Method::try_from(method).ok().unwrap();
        self
    }

    /// Overwrite the URI of this dummy request with given value.
    ///
    /// # Panics
    /// This method will panic if the parameter is invalid HTTP method.
    pub fn uri<U>(&mut self, uri: U) -> &mut ClientRequest<'a, E>
    where
        Uri: HttpTryFrom<U>,
    {
        *self.request.uri_mut() = Uri::try_from(uri).ok().unwrap();
        self
    }

    /// Append the given header entry into this dummy request.
    ///
    /// # Panics
    /// This method will panic if the given header name or value is invalid.
    pub fn header<K, V>(&mut self, name: K, value: V) -> &mut ClientRequest<'a, E>
    where
        HeaderName: HttpTryFrom<K>,
        HeaderValue: HttpTryFrom<V>,
    {
        let name = HeaderName::try_from(name).ok().unwrap();
        let value = HeaderValue::try_from(value).ok().unwrap();
        self.request.headers_mut().insert(name, value);
        self
    }

    /// Overwrite the message body of this dummy request with given instance.
    pub fn body(&mut self, body: RequestBody) -> &mut ClientRequest<'a, E> {
        self.body = Some(body);
        self
    }

    fn take(&mut self) -> ClientRequest<'a, E> {
        mem::replace(
            self,
            ClientRequest {
                client: self.client,
                request: http::Request::new(()),
                body: None,
            },
        )
    }

    /// Apply this dummy request to the associated endpoint and get its response.
    pub fn run(&mut self) -> Result<E::Output, Error> {
        let ClientRequest { client, request, body } = self.take();

        let input = Input::new(request);
        let body = body.unwrap_or_else(RequestBody::empty);

        let apply = client.endpoint.apply_request(&input, body);
        let task = TestFuture { apply, input };

        // TODO: replace with futures::executor
        task.wait().expect("Apply never fails")
    }
}

struct TestFuture<T> {
    apply: ApplyRequest<T>,
    input: Input,
}

impl<T: Task> Future for TestFuture<T> {
    type Item = Result<T::Output, Error>;
    type Error = Never;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        match self.apply.poll_ready(&self.input) {
            Poll::Pending => Ok(Async::NotReady),
            Poll::Ready(ready) => Ok(Async::Ready(ready)),
        }
    }
}
