extern crate finchers;

use finchers::Endpoint;
use finchers::endpoint::{param, segment, EndpointError};
use finchers::endpoint::method::get;
use finchers::test::{run_test, TestCase};

fn main() {
    let endpoint = || get(segment("foo").with(segment("bar")).with(param::<u64>()));

    let input = TestCase::get("/foo/bar/42");
    let output = run_test(endpoint(), input);
    assert_eq!(output.unwrap().unwrap(), 42);

    let input = TestCase::get("/foo/bar/a_str");
    let output = run_test(endpoint(), input);
    assert_eq!(output.unwrap_err(), EndpointError::Skipped);
}
