use finchers;
use finchers::prelude::*;
use finchers::rt::testing;

fn foo() -> impl_endpoint!(Output = (u32,)) {
    endpoint::unit().map(|| 42).into()
}

#[test]
fn test_send_endpoint() {
    let mut runner = testing::runner(foo().with_output::<(u32,)>());
    assert_matches!(runner.apply("/"), Ok(42));
}

#[test]
fn smoke_test() {
    let endpoint = foo()
        .with_output::<(u32,)>()
        .map(|id: u32| format!("{}", id));

    drop(move || {
        finchers::rt::launch(endpoint)
            .serve_http("127.0.0.1:4000")
            .expect("failed to start the server");
    });
}
