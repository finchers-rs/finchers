extern crate finchers;

use finchers::Endpoint;
use finchers::endpoint::{EndpointError, FromPath};
use finchers::endpoint::method::get;
use finchers::test::{run_test, TestCase};

fn main() {
    let endpoint = || get("foo".with("bar").with(u64::PATH));

    let input = TestCase::get("/foo/bar/42");
    let output = run_test(endpoint(), input);
    assert_eq!(output.unwrap().unwrap(), 42);

    let input = TestCase::get("/foo/bar/a_str");
    let output = run_test(endpoint(), input);
    assert_eq!(output.unwrap_err(), EndpointError::Skipped);
}
