#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate finchers;
#[macro_use]
extern crate serde;
extern crate http;

mod api;
mod app;
mod db;

fn main() {
    let app = app::new();
    let endpoint = api::build_endpoint(&app);
    finchers::run(endpoint);
}
