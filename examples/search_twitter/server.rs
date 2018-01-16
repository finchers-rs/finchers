use std::io;
use std::net::SocketAddr;
use futures::{Future, Stream};
use hyper::{Error, Request, Response};
use hyper::server::{Http, NewService};
use tokio_core::reactor::{Core, Handle};

#[derive(Debug)]
pub struct Server {
    core: Core,
}

impl Server {
    pub fn new() -> io::Result<Self> {
        Ok(Server { core: Core::new()? })
    }

    pub fn handle(&self) -> Handle {
        self.core.handle()
    }

    pub fn serve<S>(&mut self, addr: &SocketAddr, new_service: S) -> Result<(), Error>
    where
        S: NewService<Request = Request, Response = Response, Error = Error> + 'static,
    {
        let mut http = Http::new();
        http.pipeline(true);

        let serves = http.serve_addr_handle(addr, &self.handle(), new_service)?;

        self.core.run(serves.for_each(|conn| conn.map(|_| ())))
    }
}
