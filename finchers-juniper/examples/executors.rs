extern crate finchers;
extern crate finchers_juniper;
extern crate futures; // 0.1
extern crate futures_cpupool;
#[macro_use]
extern crate juniper;
#[macro_use]
extern crate log;
extern crate pretty_env_logger;

use finchers::endpoint::syntax;
use finchers::prelude::*;
use finchers_juniper::execute;

use futures_cpupool::CpuPool;
use juniper::{EmptyMutation, RootNode};
use std::sync::Arc;

struct MyContext {
    _priv: (),
}

impl juniper::Context for MyContext {}

struct Query;

graphql_object!(Query: MyContext |&self| {
    field apiVersion() -> &str {
        "1.0"
    }
});

fn main() {
    pretty_env_logger::init();

    let schema = Arc::new(RootNode::new(Query, EmptyMutation::<MyContext>::new()));

    let current_thread_endpoint = syntax::segment("current")
        .map(|| MyContext { _priv: () })
        .wrap(execute::current_thread(schema.clone()));

    let nonblocking_endpoint = syntax::segment("nonblocking")
        .map(|| MyContext { _priv: () })
        .wrap(execute::nonblocking(schema.clone()));

    let cpupool_endpoint = syntax::segment("cpupool")
        .map(|| MyContext { _priv: () })
        .wrap(execute::with_spawner(
            schema.clone(),
            CpuPool::new_num_cpus(),
        ));

    let endpoint = current_thread_endpoint
        .or(nonblocking_endpoint)
        .or(cpupool_endpoint);

    info!("Listening on http://127.0.0.1:4000/");
    finchers::server::start(endpoint)
        .serve("127.0.0.1:4000")
        .unwrap_or_else(|err| error!("{}", err));
}
