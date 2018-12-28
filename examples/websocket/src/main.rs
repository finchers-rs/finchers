use finchers::prelude::*;
use futures::prelude::*;
use http::Response;

use finchers_tungstenite::{ws, Ws, WsTransport};
use tungstenite::error::Error as WsError;
use tungstenite::Message;

fn on_upgrade(stream: WsTransport) -> impl Future<Item = (), Error = ()> {
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
}

fn main() {
    pretty_env_logger::init();

    let index = finchers::path!(/).map(|| {
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

    let ws_endpoint = finchers::path!(/ "ws" /).and(ws()).map(|ws: Ws| {
        log::info!("accepted a WebSocket request");
        ws.on_upgrade(on_upgrade)
    });

    let endpoint = index.or(ws_endpoint);

    log::info!("Listening on http://127.0.0.1:4000");
    finchers::server::start(endpoint)
        .serve("127.0.0.1:4000")
        .unwrap_or_else(|e| log::error!("{}", e));
}
