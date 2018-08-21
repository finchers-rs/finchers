use finchers::endpoint::EndpointExt;
use finchers::endpoints::body;
use finchers::input::body::FromBody;
use finchers::input::Input;
use finchers::local;

use bytes::Bytes;
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
    let endpoint = body::parse::<SomeData>().map(|SomeData(data)| data);

    let message = "The quick brown fox jumps over the lazy dog";

    assert_matches!(
        local::post("/").body(message).apply(&endpoint),
        Ok((ref s,)) if s == message
    );
}
