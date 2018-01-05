#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate finchers;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

mod service;
#[macro_use]
mod api;

use finchers::Application;

fn main() {
    let endpoint = build_endpoint!();
    Application::from_endpoint(endpoint).run();
}
