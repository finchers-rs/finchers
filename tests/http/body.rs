use bytes::Bytes;
use finchers_core::endpoint::ext::EndpointExt;
use finchers_core::http::body::{body, FromBody};
use finchers_core::input::{Input, RequestBody};
use finchers_runtime::local::Client;

#[derive(Default)]
struct SomeData(String);

impl FromBody for SomeData {
    type Error = ();

    fn from_body(bytes: Bytes, _: &Input) -> Result<Self, Self::Error> {
        String::from_utf8(bytes.to_vec())
            .map(SomeData)
            .map_err(|_| ())
    }
}

#[test]
fn test_body_1() {
    let endpoint = body::<SomeData>()
        .map(|res| res.unwrap_or_default())
        .map(|SomeData(data)| data)
        .as_t::<String>();

    const MESSAGE: &str = "The quick brown fox jumps over the lazy dog";

    let client = Client::new(&endpoint);
    let outcome = client.post("/").body(RequestBody::once(MESSAGE)).run();

    assert_eq!(outcome, Some(MESSAGE.into()));
}
