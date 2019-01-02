use finchers;
use finchers::endpoint::{syntax, ApplyContext, ApplyResult, Endpoint, Wrapper};
use finchers::test;

use matches::assert_matches;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[derive(Debug, Default)]
struct Wrap {
    counter: Arc<AtomicUsize>,
}

impl<E: Endpoint> Wrapper<E> for Wrap {
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

impl<E: Endpoint> Endpoint for Wrapped<E> {
    type Output = E::Output;
    type Future = E::Future;

    fn apply(&self, cx: &mut ApplyContext<'_>) -> ApplyResult<Self::Future> {
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
    let endpoint = syntax::verb::get().wrap(wrapper);
    let mut runner = test::runner(endpoint);

    assert_matches!(runner.apply_raw("/"), Ok(..));
    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

#[test]
#[ignore]
fn compiletest_launch_wrapped_endpoint() {
    let wrapper = Wrap {
        counter: Arc::new(AtomicUsize::new(0)),
    };
    let endpoint = syntax::verb::get().wrap(wrapper);

    finchers::server::start(endpoint)
        .serve("127.0.0.1:4000")
        .unwrap();
}
