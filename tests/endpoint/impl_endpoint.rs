use finchers;
use finchers::prelude::*;
use finchers::test;

fn foo() -> impl for<'a> Endpoint<'a, Output = (u32,), Future = impl Send + 'a> {
    endpoint::unit().map(|| 42)
}

#[test]
fn test_send_endpoint() {
    let mut runner = test::runner(foo().with_output::<(u32,)>());
    assert_matches!(runner.apply("/"), Ok(42));
}

#[test]
#[ignore]
fn compiletest() {
    let endpoint = foo()
        .with_output::<(u32,)>()
        .map(|id: u32| format!("{}", id));

    finchers::server::start(endpoint)
        .serve("127.0.0.1:4000")
        .unwrap();
}
