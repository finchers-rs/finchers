extern crate finchers_core;
extern crate finchers_endpoint;
extern crate futures;
extern crate http;

use futures::Future;
use http::{HttpTryFrom, Method, Request, Uri};

use finchers_core::input::{BodyStream, Input};
use finchers_endpoint::apply::apply;
use finchers_endpoint::{Endpoint, Error};

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
            request: Ok(Request::new(Default::default())),
        };
        client.modify(|request| {
            Method::try_from(method).map(|method| {
                *request.method_mut() = method;
            })
        });
        client.modify(|request| {
            Uri::try_from(uri).map(|uri| {
                *request.uri_mut() = uri;
            })
        });
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
    request: http::Result<Request<BodyStream>>,
}

impl<'a, E: Endpoint> ClientRequest<'a, E> {
    fn modify<F, R>(&mut self, f: F)
    where
        F: FnOnce(&mut Request<BodyStream>) -> Result<(), R>,
        R: Into<http::Error>,
    {
        if self.request.is_ok() {
            if let Err(err) = f(self.request.as_mut().unwrap()) {
                self.request = Err(err.into());
            }
        }
    }

    pub fn body<B>(mut self, body: B) -> ClientRequest<'a, E>
    where
        B: Into<BodyStream>,
    {
        if let Ok(ref mut request) = self.request {
            *request.body_mut() = body.into();
        }
        self
    }

    pub fn run(self) -> http::Result<Result<E::Item, Error>> {
        let ClientRequest { client, request } = self;
        let input: Input = request?.into();
        let f = apply(&client.endpoint, input);
        // TODO: replace with futures::executor
        Ok(f.wait())
    }
}
