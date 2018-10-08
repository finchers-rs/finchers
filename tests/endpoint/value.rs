use finchers;
use finchers::prelude::*;

#[test]
fn smoke_by_ref() {
    let endpoint = path!(@get / u32)
        .and(endpoint::by_ref(String::from("Hello, world")))
        .and(endpoints::body::text())
        .and_then(|id: u32, s: &String, body: String| {
            Ok(format!("id={}, s={}, body={}", id, s, body))
        });

    drop(move || finchers::server::start(endpoint).serve("127.0.0.1:4000"));
}
