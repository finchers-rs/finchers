use finchers::prelude::*;
use finchers_tungstenite::{Message, WsError, WsTransport};
use futures::prelude::*;
use http::Response;

fn main() -> izanami::Result<()> {
    std::env::set_var("RUST_LOG", "server=info");
    pretty_env_logger::init();

    let index = finchers::path!("/").map(|| {
        Response::builder()
            .header("content-type", "text/html; charset=utf-8")
            .body(
                r#"<!doctype html>
                <html>
                  <head>
                    <meta charset="utf-8">
                    <title>Index</title>
                  </head>
                  <body>
                  </body>
                </html>
                "#,
            )
            .unwrap()
    });

    let ws_endpoint = finchers::path!(@get "/ws") //
        .and(finchers_tungstenite::ws(
            |stream: WsTransport<izanami::RequestBody>| {
                let (tx, rx) = stream.split();
                rx.filter_map(|m| {
                    log::info!("Message from client: {:?}", m);
                    match m {
                        Message::Ping(p) => Some(Message::Pong(p)),
                        Message::Pong(..) => None,
                        m => Some(m),
                    }
                })
                .forward(tx)
                .map(|_| ())
                .map_err(|e| match e {
                    WsError::ConnectionClosed(..) => log::info!("connection is closed"),
                    e => log::error!("error during handling WebSocket connection: {}", e),
                })
            },
        ));

    let endpoint = index.or(ws_endpoint);

    log::info!("Listening on http://127.0.0.1:5000");
    izanami::Server::bind(std::net::SocketAddr::from(([127, 0, 0, 1], 5000)))
        .start(endpoint.into_service())
}
