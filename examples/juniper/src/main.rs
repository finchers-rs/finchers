mod schema;

use finchers::prelude::*;
use finchers::server::middleware::log::stdlog;

use crate::schema::{create_schema, Context};

fn main() {
    let schema = create_schema();

    let acquire_context = endpoint::unit().map(|| Context::default());

    let graphql_endpoint = finchers::path!(/ "graphql")
        .and(acquire_context)
        .wrap(finchers_juniper::execute::nonblocking(schema));

    let index_page = finchers::path!(@get /).and(finchers_juniper::graphiql_source("/graphql"));

    let endpoint = index_page.or(graphql_endpoint);

    std::env::set_var("RUST_LOG", "info");
    pretty_env_logger::init();

    log::info!("Listening on http://127.0.0.1:4000");
    finchers::server::start(endpoint)
        .with_middleware(stdlog(log::Level::Info, module_path!()))
        .serve("127.0.0.1:4000")
        .unwrap_or_else(|e| log::error!("{}", e));
}
