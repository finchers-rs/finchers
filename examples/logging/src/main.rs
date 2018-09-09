use finchers::endpoint::EndpointExt;
use finchers::output::status::Created;
use finchers::{path, routes};

use finchers::endpoints::logging::{logging_fn, Info};
use log::info;

fn main() {
    std::env::set_var("RUST_LOG", "info");
    pretty_env_logger::init();

    let endpoint = routes![
        path!(@get / "index").map(|| "Index page"),
        path!(@get / "created").map(|| Created("created")),
    ];

    let endpoint = endpoint.with(logging_fn(|info: Info| {
        info!(
            "{} {} -> {} ({:?})",
            info.input.method(),
            info.input.uri(),
            info.status,
            info.start.elapsed()
        );
    }));

    info!("Listening on http://127.0.0.1:4000...");
    finchers::launch(endpoint).start("127.0.0.1:4000");
}
