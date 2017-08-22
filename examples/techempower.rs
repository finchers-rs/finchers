extern crate finchers;
extern crate hyper;
extern crate num_cpus;
#[macro_use]
extern crate serde_derive;
extern crate tokio_proto;

use finchers::{Endpoint, Json};
use hyper::server::Http;
use tokio_proto::TcpServer;
use std::sync::Arc;

#[derive(Serialize)]
struct Message {
    message: &'static str,
}

fn main() {
    let endpoint = {
        let json = "json".map(|_| {
            Json(Message {
                message: "Hello, World!",
            })
        });

        let plaintext = "plaintext".map(|_| "Hello, World!");

        json.or(plaintext)
    };

    let service = Arc::new(endpoint).into_service();

    let addr = "0.0.0.0:8080".parse().unwrap();
    let proto = Http::new();
    let mut srv = TcpServer::new(proto, addr);
    srv.threads(num_cpus::get());
    srv.serve(move || Ok(service.clone()));
}
