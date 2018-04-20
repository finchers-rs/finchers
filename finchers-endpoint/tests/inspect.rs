extern crate finchers_core;
extern crate finchers_endpoint;
extern crate finchers_test;

use finchers_endpoint::{just, EndpointExt};
use finchers_test::Client;

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

#[test]
fn test_inspect() {
    let count = Arc::new(AtomicUsize::new(0));
    let endpoint = just("Foo").inspect({
        let count = count.clone();
        move |_| {
            count.store(42, Ordering::Relaxed);
        }
    });
    let client = Client::new(endpoint);

    let outcome = client.get("/").run();
    assert_eq!(outcome.and_then(Result::ok), Some("Foo"));
    assert_eq!(count.load(Ordering::Relaxed), 42);
}
