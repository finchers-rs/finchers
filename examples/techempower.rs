extern crate finchers;
extern crate num_cpus;
#[macro_use]
extern crate serde_derive;

use finchers::{Endpoint, Json};
use finchers::server::Server;

#[derive(Serialize)]
struct Message {
    message: &'static str,
}

fn main() {
    let endpoint = |_: &_| {
        let json = "json".map(|_| {
            Json(Message {
                message: "Hello, World!",
            })
        });

        let plaintext = "plaintext".map(|_| "Hello, World!");

        json.or(plaintext)
    };

    Server::new(endpoint)
        .num_workers(num_cpus::get())
        .bind("0.0.0.0:8080")
        .run_http()
}
