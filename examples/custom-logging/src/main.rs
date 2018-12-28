use finchers::output::status::Created;
use finchers::prelude::*;
use finchers::server::middleware::log::log;
use finchers::{path, routes};

use slog::Logger;

fn main() {
    let endpoint = routes![
        path!(@get / "index").map(|| "Index page"),
        path!(@get / "created").map(|| Created("created")),
    ];

    let logger = build_logger();
    let log_middleware = log(logging::Slog {
        logger: logger.new(slog::o!("local_addr" => "http://127.0.0.1:4000")),
    });

    finchers::server::start(endpoint)
        .with_middleware(log_middleware)
        .serve("127.0.0.1:4000")
        .unwrap_or_else(|e| slog::error!(logger, "{}", e));
}

fn build_logger() -> Logger {
    use sloggers::terminal::{Destination, TerminalLoggerBuilder};
    use sloggers::types::{Format, Severity};
    use sloggers::Build;

    let mut builder = TerminalLoggerBuilder::new();
    builder.level(Severity::Debug);
    builder.destination(Destination::Stdout);
    builder.format(Format::Full);
    builder.build().expect("failed to construct a Logger")
}

mod logging {
    use finchers::server::middleware::log::{Logger, Logging};
    use http::{Request, Response};
    use std::time::Instant;

    #[derive(Clone)]
    pub struct Slog {
        pub logger: slog::Logger,
    }

    impl Logger for Slog {
        type Instance = SlogInstance;

        fn start<T>(&self, request: &Request<T>) -> Self::Instance {
            let start = Instant::now();
            SlogInstance {
                logger: self.logger.new(slog::o! {
                    "request_method" => request.method().to_string(),
                    "request_uri" => request.uri().to_string(),
                }),
                start,
            }
        }
    }

    pub struct SlogInstance {
        logger: slog::Logger,
        start: Instant,
    }

    impl Logging for SlogInstance {
        fn finish<T>(self, response: &Response<T>) {
            slog::info!(self.logger, "response";
                "response_status" => response.status().to_string(),
                "response_time" => format!("{:?}", self.start.elapsed()),
            );
        }
    }
}
