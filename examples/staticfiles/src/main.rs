#![feature(async_await, await_macro, pin, futures_api)]

mod fs;
mod named_file;

use finchers::endpoint::EndpointExt;
use finchers::{route, routes};

fn main() -> finchers::rt::LaunchResult<()> {
    let endpoint = routes![
        route!(@get /).and(fs::file("./Cargo.toml")),
        route!(@get / "public").and(fs::dir("./static")),
        route!(@get).map(|| "Not found"),
    ];

    finchers::rt::launch(endpoint)
}
