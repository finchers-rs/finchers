use std::io;
use std::string::ToString;
use std::sync::Arc;
use hyper;
use hyper::server::{NewService, Service};

use endpoint::{Endpoint, Outcome};
use response::{DefaultResponder, HttpStatus};
use service::{EndpointServiceExt, FinchersService};

use super::{Http, Tcp, TcpBackend, Worker};
use super::backend::DefaultBackend;

/// The launcher of HTTP application.
#[derive(Debug)]
pub struct Application<S, B>
where
    S: NewService<Request = hyper::Request, Response = hyper::Response, Error = hyper::Error> + Clone + 'static,
    B: TcpBackend,
{
    new_service: S,
    http: Http,
    tcp: Tcp<B>,
    worker: Worker,
}

impl<S, B> Application<S, B>
where
    S: NewService<Request = hyper::Request, Response = hyper::Response, Error = hyper::Error> + Clone + 'static,
    B: TcpBackend,
{
    /// Create a new launcher from given service and TCP backend.
    pub fn new(new_service: S, backend: B) -> Self {
        Application {
            new_service,
            http: Http::default(),
            worker: Worker::default(),
            tcp: Tcp {
                addrs: vec![],
                backend,
            },
        }
    }

    /// Returns a mutable reference of the service.
    pub fn new_service(&mut self) -> &mut S {
        &mut self.new_service
    }

    /// Returns a mutable reference of the HTTP configuration
    pub fn http(&mut self) -> &mut Http {
        &mut self.http
    }

    /// Returns a mutable reference of the TCP configuration
    pub fn tcp(&mut self) -> &mut Tcp<B> {
        &mut self.tcp
    }

    /// Returns a mutable reference of the worker configuration
    pub fn worker(&mut self) -> &mut Worker {
        &mut self.worker
    }

    pub(super) fn deconstruct(self) -> (S, Http, Tcp<B>, Worker) {
        (self.new_service, self.http, self.tcp, self.worker)
    }
}

impl<S> Application<ConstService<S>, DefaultBackend>
where
    S: Service<Request = hyper::Request, Response = hyper::Response, Error = hyper::Error>,
{
    #[allow(missing_docs)]
    pub fn from_service(service: S) -> Self {
        Self::new(
            ConstService {
                service: Arc::new(service),
            },
            Default::default(),
        )
    }
}

impl<E, T> Application<ConstService<FinchersService<E, DefaultResponder, T>>, DefaultBackend>
where
    E: Endpoint,
    E::Item: Into<Outcome<T>>,
    T: HttpStatus + ToString,
{
    #[allow(missing_docs)]
    pub fn from_endpoint(endpoint: E) -> Self {
        Self::from_service(endpoint.into_service())
    }
}

impl<S, B> Application<S, B>
where
    S: NewService<Request = hyper::Request, Response = hyper::Response, Error = hyper::Error> + Clone + 'static,
    B: TcpBackend,
    S: Send + Sync,
    B: Send + Sync + 'static,
{
    /// Start the HTTP server with given configurations
    #[inline]
    pub fn run(self) {
        super::worker::start_multi_threaded(self).expect("error from hyper")
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct ConstService<S: Service> {
    service: Arc<S>,
}

impl<S: Service> Clone for ConstService<S> {
    fn clone(&self) -> Self {
        ConstService {
            service: self.service.clone(),
        }
    }
}

impl<S: Service> NewService for ConstService<S> {
    type Request = S::Request;
    type Response = S::Response;
    type Error = S::Error;
    type Instance = Arc<S>;

    fn new_service(&self) -> io::Result<Self::Instance> {
        Ok(self.service.clone())
    }
}
