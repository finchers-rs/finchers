extern crate finchers;

use finchers::Application;
use finchers::endpoint::ok;

fn main() {
    let endpoint = ok::<&str, ()>("Hello, Finchers");
    Application::from_endpoint(endpoint).run();
}
