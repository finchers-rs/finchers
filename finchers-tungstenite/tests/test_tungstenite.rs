use {finchers::prelude::*, http::Request, matches::assert_matches};

#[test]
fn version_sync() {
    version_sync::assert_html_root_url_updated!("src/lib.rs");
}

#[test]
fn test_handshake() -> izanami::Result<()> {
    let mut server = izanami::test::server({
        finchers_tungstenite::ws(|stream| {
            drop(stream);
            futures::future::ok(())
        })
        .into_service()
    })?;

    let response = server.perform(
        Request::get("/")
            .header("host", "localhost:4000")
            .header("connection", "upgrade")
            .header("upgrade", "websocket")
            .header("sec-websocket-version", "13")
            .header("sec-websocket-key", "dGhlIHNhbXBsZSBub25jZQ=="),
    )?;

    assert_eq!(response.status().as_u16(), 101);
    //assert!(response.body().is_upgraded());
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

    Ok(())
}
