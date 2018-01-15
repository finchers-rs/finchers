extern crate egg_mode;
#[macro_use]
extern crate finchers;
#[macro_use]
extern crate futures;
extern crate hyper;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate tokio_core;

mod common;
#[macro_use]
mod endpoint;
mod handler;
mod server;

use finchers::responder::DefaultResponder;
use finchers::service::FinchersService;
use std::rc::Rc;

use handler::SearchTwitterHandler;
use server::Server;

fn main() {
    let mut server = Server::new().unwrap();

    let endpoint = Rc::new(build_endpoint!());

    let token = common::retrieve_access_token(server.reactor());
    let handler = SearchTwitterHandler::new(token, server.handle());

    let service = FinchersService::new(endpoint, handler, DefaultResponder::default());

    let addr = "0.0.0.0:4000".parse().unwrap();
    println!("Listening on {}...", addr);
    let _ = server.serve(&addr, move || Ok(service.clone()));
}
