use finchers::endpoint::syntax::verb;
use finchers::endpoint::EndpointExt;
use finchers::endpoints::fs;
use finchers::{path, routes};

fn main() {
    let endpoint = routes![
        path!(@get /).and(fs::file("./Cargo.toml")),
        path!(@get / "public").and(fs::dir("./static")),
        verb::get().map(|| "Not found"),
    ];

    finchers::launch(endpoint).start("127.0.0.1:5000")
}
