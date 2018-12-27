#[macro_use]
extern crate finchers;
extern crate finchers_tungstenite;
extern crate futures;
extern crate http;
extern crate pretty_env_logger;
#[macro_use]
extern crate log;

use finchers::prelude::*;
use futures::prelude::*;
use http::Response;

use finchers_tungstenite::{Message, Ws, WsError, WsTransport};

fn on_upgrade(stream: WsTransport) -> impl Future<Item = (), Error = ()> {
    let (tx, rx) = stream.split();
    rx.filter_map(|m| {
        info!("Message from client: {:?}", m);
        match m {
            Message::Ping(p) => Some(Message::Pong(p)),
            Message::Pong(..) => None,
            m => Some(m),
        }
    }).forward(tx)
    .map(|_| ())
    .map_err(|e| match e {
        WsError::ConnectionClosed(..) => info!("connection is closed"),
        e => error!("error during handling WebSocket connection: {}", e),
    })
}

fn main() {
    std::env::set_var("RUST_LOG", "server=info");
    pretty_env_logger::init();

    let index = path!(/).map(|| {
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
            ).unwrap()
    });

    let ws_endpoint = path!(/ "ws" /)
        .and(finchers_tungstenite::ws())
        .map(|ws: Ws| {
            info!("accepted a WebSocket request");
            ws.on_upgrade(on_upgrade)
        });

    let endpoint = index.or(ws_endpoint);

    info!("Listening on http://127.0.0.1:5000");
    finchers::server::start(endpoint)
        .serve("127.0.0.1:5000")
        .unwrap_or_else(|err| error!("{}", err))
}
