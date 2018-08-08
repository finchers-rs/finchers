//! A testing framework for Finchers.

use futures::{future, Async, Future as _Future};
use http::header::{HeaderName, HeaderValue};
use http::{self, HttpTryFrom, Method, Request, Uri};
use std::mem;

use finchers_core::endpoint::EndpointBase;
use finchers_core::future::Poll;
use finchers_core::input::RequestBody;
use finchers_core::{Input, Never};

use apply::apply_request;

/// A wrapper struct of an endpoint which adds the facility for testing.
#[derive(Debug)]
pub struct Client<E: EndpointBase> {
    endpoint: E,
}

impl<E: EndpointBase> Client<E> {
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
            request: Request::new(RequestBody::empty()),
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

impl<E: EndpointBase> Client<E> {
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
pub struct ClientRequest<'a, E: EndpointBase + 'a> {
    client: &'a Client<E>,
    request: Request<RequestBody>,
}

impl<'a, E: EndpointBase> ClientRequest<'a, E> {
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
        mem::replace(self.request.body_mut(), body);
        self
    }

    fn take(&mut self) -> ClientRequest<'a, E> {
        mem::replace(
            self,
            ClientRequest {
                client: self.client,
                request: http::Request::new(RequestBody::empty()),
            },
        )
    }

    /// Apply this dummy request to the associated endpoint and get its response.
    pub fn run(&mut self) -> Option<E::Output> {
        let ClientRequest { client, request } = self.take();

        let mut input = Input::new(request);
        let mut apply = apply_request(&client.endpoint, &input);

        let future = future::poll_fn(move || match apply.poll_ready(&mut input) {
            Poll::Pending => Ok(Async::NotReady) as Result<_, Never>,
            Poll::Ready(ready) => Ok(Async::Ready(ready)),
        });

        // TODO: replace with futures::executor
        future.wait().expect("Apply never fails")
    }
}
