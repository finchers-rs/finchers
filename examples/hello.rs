extern crate finchers;

use finchers::Application;
use finchers::endpoint::ok;

fn handler<T>(value: T) -> Result<T, ()> {
    Ok(value)
}

fn main() {
    let endpoint = ok::<&str, ()>("Hello, Finchers");
    Application::new(endpoint, handler).run();
}
