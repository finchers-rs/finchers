extern crate base64;
extern crate finchers;

mod basic_auth;

use basic_auth::{basic_auth, BasicAuth};
use finchers::EndpointExt;

fn main() {
    let endpoint = basic_auth()
        .map(|BasicAuth { username, password }| format!("Hello, \"{}\" (password={:?})", username, password));

    finchers::run(endpoint);
}
