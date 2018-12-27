extern crate finchers;
extern crate finchers_tungstenite;
extern crate futures;
extern crate http;
#[macro_use]
extern crate matches;

use http::Request;

use finchers::prelude::*;
use finchers::test;
use finchers_tungstenite::Ws;

#[test]
fn test_handshake() {
    let mut runner = test::runner({
        finchers_tungstenite::ws().map({
            move |ws: Ws| {
                ws.on_upgrade(|stream| {
                    drop(stream);
                    futures::future::ok(())
                })
            }
        })
    });

    let response = runner
        .perform(
            Request::get("/")
                .header("host", "localhost:4000")
                .header("connection", "upgrade")
                .header("upgrade", "websocket")
                .header("sec-websocket-version", "13")
                .header("sec-websocket-key", "dGhlIHNhbXBsZSBub25jZQ=="),
        ).unwrap();

    assert_eq!(response.status().as_u16(), 101);
    assert!(response.body().is_upgraded());
    assert_matches!(
        response.headers().get("connection"),
        Some(h) if h.to_str().unwrap().to_lowercase() == "upgrade"
    );
    assert_matches!(
        response.headers().get("upgrade"),
        Some(h) if h.to_str().unwrap().to_lowercase() == "websocket"
    );
    assert_matches!(
        response.headers().get("sec-websocket-accept"),
        Some(h) if h == "s3pPLMBiTxaQ9kYGzzhZRbK+xOo="
    );
}
