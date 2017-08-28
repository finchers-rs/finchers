extern crate finchers;
extern crate num_cpus;
#[macro_use]
extern crate serde_derive;

use finchers::{Endpoint, Json};
use finchers::endpoint::method::get;
use finchers::server::Server;
use finchers::util::either::Either2;

#[derive(Serialize)]
struct Message {
    message: &'static str,
}

fn main() {
    let endpoint = |_: &_| {
        let json = get("json").map(|_| {
            Json(Message {
                message: "Hello, World!",
            })
        });

        let plaintext = get("plaintext").map(|_| "Hello, World!");

        (json.map(Either2::E1)).or(plaintext.map(Either2::E2))
    };

    Server::new(endpoint)
        .num_workers(num_cpus::get())
        .bind("0.0.0.0:8080")
        .run_http()
}
