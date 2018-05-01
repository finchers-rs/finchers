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

#[derive(Debug)]
pub struct Client<E: Endpoint> {
    endpoint: E,
}

macro_rules! impl_constructors {
    ($($METHOD:ident => $name:ident,)*) => {$(
        pub fn $name<'a, U>(&'a self, uri: U) -> ClientRequest<'a, E>
        where
            Uri: HttpTryFrom<U>,
        {
            self.request(Method::$METHOD, uri)
        }
    )*};
}

impl<E: Endpoint> Client<E> {
    pub fn new(endpoint: E) -> Client<E> {
        Client { endpoint }
    }

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

    impl_constructors! {
        GET => get,
        POST => post,
        PUT => put,
        HEAD => head,
        DELETE => delete,
        PATCH => patch,
    }
}

#[derive(Debug)]
pub struct ClientRequest<'a, E: Endpoint + 'a> {
    client: &'a Client<E>,
    request: Request<()>,
    body: Option<RequestBody>,
}

impl<'a, E: Endpoint> ClientRequest<'a, E> {
    pub fn method<M>(&mut self, method: M) -> &mut ClientRequest<'a, E>
    where
        Method: HttpTryFrom<M>,
    {
        *self.request.method_mut() = Method::try_from(method).ok().unwrap();
        self
    }

    pub fn uri<U>(&mut self, uri: U) -> &mut ClientRequest<'a, E>
    where
        Uri: HttpTryFrom<U>,
    {
        *self.request.uri_mut() = Uri::try_from(uri).ok().unwrap();
        self
    }

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

    pub fn run(&mut self) -> Option<Result<E::Output, Error>> {
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
    type Item = Option<Result<T::Output, Error>>;
    type Error = Never;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        match self.apply.poll_ready(&self.input) {
            Poll::Pending => Ok(Async::NotReady),
            Poll::Ready(ready) => Ok(Async::Ready(ready)),
        }
    }
}
