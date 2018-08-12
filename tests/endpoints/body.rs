use bytes::Bytes;
use finchers_core::endpoint::EndpointExt;
use finchers_core::endpoints::body::body;
use finchers_core::input::{FromBody, Input, RequestBody};
use finchers_core::local;
use std::mem::PinMut;
use std::string::FromUtf8Error;

#[derive(Default)]
struct SomeData(String);

impl FromBody for SomeData {
    type Error = FromUtf8Error;

    fn from_body(bytes: Bytes, _: PinMut<Input>) -> Result<Self, Self::Error> {
        String::from_utf8(bytes.to_vec()).map(SomeData)
    }
}

#[test]
fn test_body_1() {
    let endpoint = body::<SomeData>().map(|SomeData(data)| data);

    let message = "The quick brown fox jumps over the lazy dog";

    assert_eq!(
        local::post("/")
            .body(RequestBody::once(message))
            .apply(&endpoint),
        Some(Ok((message.into(),))),
    );
}
