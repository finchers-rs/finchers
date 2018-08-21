#![feature(rust_2018_preview)]

extern crate finchers;

use finchers::endpoint::EndpointExt;
use finchers::endpoints::fs;
use finchers::{route, routes};

fn main() {
    let endpoint = routes![
        route!(@get /).and(fs::file("./Cargo.toml")),
        route!(@get / "public").and(fs::dir("./static")),
        route!(@get).map(|| "Not found"),
    ];

    finchers::launch(endpoint).start("127.0.0.1:5000")
}
