#[macro_use]
extern crate finchers;

use finchers::{Application, Endpoint};

fn main() {
    let endpoint = endpoint!(()).assert_types::<_, ()>();
    let handler = |_| Ok("Hello, Finchers") as Result<_, ()>;

    Application::new(endpoint, handler).run();
}
