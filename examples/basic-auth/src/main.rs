extern crate base64;
extern crate finchers;
extern crate http;

mod basic_auth;

use basic_auth::{BasicAuth, Unauthorized};
use finchers::endpoint::EndpointExt;

fn main() {
    let basic_auth = {
        use finchers::endpoint::header::header;
        use finchers::endpoint::prelude::*;
        header().map_err(|_| Unauthorized).unwrap_ok()
    };

    let endpoint = basic_auth.map(|BasicAuth { username, password }| {
        format!("Hello, \"{}\" (password={:?})", username, password)
    });

    finchers::run(endpoint);
}
