#![feature(rust_2018_preview)]
#![feature(pin, arbitrary_self_types, futures_api)]

extern crate bytes;
extern crate failure;
extern crate finchers;
extern crate futures_util;
extern crate http;
extern crate serde;

macro_rules! assert_matches {
    ($e:expr, $($t:tt)+) => {
        assert_matches!(@hack match $e {
            $($t)+ => {},
            ref e => panic!("assertion failed: `{:?}` does not match `{}`", e, stringify!($($t)+)),
        })
    };
    (@hack $v:expr) =>  { $v };
}

//mod codegen;
mod endpoint;
mod endpoints;

#[test]
fn smoketest() {
    use finchers::endpoint::EndpointExt;
    use finchers::local;
    use finchers::output::status::Created;
    use finchers::output::Json;
    use finchers::route;

    let endpoint = route!(@get / "api" / "v1" / "posts" / u32).map(|id: u32| Created(Json(id)));

    let response = local::get("/api/v1/posts/42").respond(&endpoint);
    assert_eq!(response.status().as_u16(), 201);
    assert_eq!(
        response.headers().get("content-type").map(|h| h.as_bytes()),
        Some(&b"application/json"[..])
    );
    assert_eq!(response.body().to_utf8(), "42");
}
