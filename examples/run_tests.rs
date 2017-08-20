extern crate finchers;
extern crate hyper;

use finchers::Endpoint;
use finchers::combinator::path::{u64_, end_};
use finchers::combinator::method::get;
use finchers::test::{TestCase, run_test};
use hyper::Get;

fn main() {
    let endpoint = || get("foo".with("bar").with(u64_).skip(end_));

    let input = TestCase::new(Get, "/foo/bar/42").expect("invalid URI");
    let output = run_test(endpoint(), input);
    assert!(output.is_ok());
    assert_eq!(output.unwrap(), 42);

    let input = TestCase::new(Get, "/foo/bar/a_str").expect("invalid URI");
    let output = run_test(endpoint(), input);
    assert!(output.is_err());
}
