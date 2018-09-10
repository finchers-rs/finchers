// FIXME: remove this item after `slog` supports 2018-style macro imports
#[macro_use]
extern crate slog;

use finchers::endpoint::EndpointExt;
use finchers::output::status::Created;
use finchers::{path, routes};

use finchers::endpoints::logging::{logging_fn, Info};

use slog::Logger;
use sloggers::terminal::{Destination, TerminalLoggerBuilder};
use sloggers::types::Severity;
use sloggers::Build;

fn build_logger() -> Logger {
    let mut builder = TerminalLoggerBuilder::new();
    builder.level(Severity::Debug);
    builder.destination(Destination::Stdout);
    builder.build().expect("failed to construct a Logger")
}

fn main() {
    let endpoint = routes![
        path!(@get / "index").map(|| "Index page"),
        path!(@get / "created").map(|| Created("created")),
    ];

    let logger = build_logger();
    let logging = logging_fn({
        let logger = logger.clone();
        move |info: Info| {
            slog_info!(
                logger,
                "{} {} -> {} ({:?})",
                info.input.method(),
                info.input.uri(),
                info.status,
                info.start.elapsed()
            );
        }
    });

    let endpoint = endpoint.with(logging);

    slog_info!(logger, "Listening on http://127.0.0.1:4000...");
    finchers::launch(endpoint).start("127.0.0.1:4000");
}
