extern crate finchers;

use finchers::Endpoint;
use finchers::endpoint::{end_, u64_};
use finchers::endpoint::method::get;
use finchers::test::{run_test, TestCase};

fn main() {
    let endpoint = || get("foo".with("bar").with(u64_).skip(end_));

    let input = TestCase::get("/foo/bar/42");
    let output = run_test(endpoint(), input);
    assert!(output.is_ok());
    assert_eq!(output.unwrap(), 42);

    let input = TestCase::get("/foo/bar/a_str");
    let output = run_test(endpoint(), input);
    assert!(output.is_err());
}
