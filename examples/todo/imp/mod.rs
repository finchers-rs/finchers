mod service;
mod api;

use finchers::Application;

pub fn main() {
    let endpoint = api::build_endpoint();
    Application::from_endpoint(endpoint).run();
}
