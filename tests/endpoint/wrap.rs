use finchers;
use finchers::endpoint::{ApplyContext, ApplyResult, Endpoint, Wrapper};
use finchers::test;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[derive(Debug, Default)]
struct Wrap {
    counter: Arc<AtomicUsize>,
}

impl<'a, E: Endpoint<'a>> Wrapper<'a, E> for Wrap {
    type Output = E::Output;
    type Endpoint = Wrapped<E>;

    fn wrap(self, endpoint: E) -> Self::Endpoint {
        Wrapped {
            endpoint,
            counter: self.counter,
        }
    }
}

struct Wrapped<E> {
    endpoint: E,
    counter: Arc<AtomicUsize>,
}

impl<'a, E: Endpoint<'a>> Endpoint<'a> for Wrapped<E> {
    type Output = E::Output;
    type Future = E::Future;

    fn apply(&'a self, cx: &mut ApplyContext<'_>) -> ApplyResult<Self::Future> {
        self.counter.fetch_add(1, Ordering::SeqCst);
        self.endpoint.apply(cx)
    }
}

#[test]
fn test_wrap() {
    let counter = Arc::new(AtomicUsize::new(0));

    let wrapper = Wrap {
        counter: counter.clone(),
    };
    let endpoint = path!(@get /).wrap(wrapper);
    let mut runner = test::runner(endpoint);

    assert_matches!(runner.apply_raw("/"), Ok(..));
    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

#[test]
fn test_launch_wrapped_endpoint() {
    let wrapper = Wrap {
        counter: Arc::new(AtomicUsize::new(0)),
    };
    let endpoint = path!(@get /).wrap(wrapper);

    drop(move || finchers::server::start(endpoint).serve("127.0.0.1:4000"));
}
