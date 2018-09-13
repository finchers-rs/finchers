use finchers::local;
use finchers::prelude::*;

use matches::assert_matches;

fn foo() -> impl for<'a> SendEndpoint<'a, Output = (u32,)> {
    endpoint::unit().map(|| 42)
}

#[test]
fn test_send_endpoint() {
    let endpoint = foo().into_local().with_output::<(u32,)>();

    assert_matches!(local::get("/").apply(&endpoint), Ok((42,)));
}

#[test]
fn smoke_test() {
    let endpoint = foo()
        .into_local()
        .with_output::<(u32,)>()
        .map(|id: u32| format!("{}", id));

    drop(move || {
        finchers::launch(endpoint).start("127.0.0.1:4000");
    });
}
