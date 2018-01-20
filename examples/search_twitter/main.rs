extern crate egg_mode;
#[macro_use]
extern crate finchers;
#[macro_use]
extern crate futures;
extern crate hyper;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate tokio_core;

mod config;
#[macro_use]
mod endpoint;
mod error;
mod handler;
mod responder;
mod server;

use finchers::service::FinchersService;
use std::rc::Rc;

use handler::SearchTwitterHandler;
use responder::SearchTwitterResponder;
use server::Server;

fn main() {
    let token = config::retrieve_bearer_token("config.json");

    let mut server = Server::new().unwrap();

    let endpoint = Rc::new(build_endpoint!());
    let handler = SearchTwitterHandler::new(token, server.handle());
    let responder = SearchTwitterResponder::default();
    let service = FinchersService::new(endpoint, handler, responder);

    let addr = "0.0.0.0:4000".parse().unwrap();
    println!("Listening on {}...", addr);
    let _ = server.serve(&addr, move || Ok(service.clone()));
}
