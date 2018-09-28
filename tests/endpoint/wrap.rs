use finchers;
use finchers::endpoint::{ApplyContext, ApplyResult, Endpoint, Wrapper};
use finchers::local;

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

    assert_matches!(local::get("/").apply(&endpoint), Ok(..));

    assert_eq!(counter.load(Ordering::SeqCst), 1);

    drop(move || finchers::launch(endpoint).start("127.0.0.1:4000"));
}
