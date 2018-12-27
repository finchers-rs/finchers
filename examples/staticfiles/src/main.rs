use finchers::prelude::*;
use finchers::{path, routes};

fn main() {
    let endpoint = routes![
        path!(@get /).and(endpoints::fs::file("./Cargo.toml")),
        path!(@get / "public").and(endpoints::fs::dir("./static")),
        endpoint::syntax::verb::get().map(|| "Not found"),
    ];

    finchers::server::start(endpoint)
        .serve("127.0.0.1:5000")
        .unwrap_or_else(|e| eprintln!("{}", e));
}
