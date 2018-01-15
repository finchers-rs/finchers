#[macro_use]
extern crate finchers;

use finchers::Application;

fn main() {
    let endpoint = endpoint!(() => <_, ()>);
    let handler = |_| Ok("Hello, Finchers") as Result<_, ()>;

    Application::new(endpoint, handler).run();
}
