// FIXME: remove it as soon as the rustc version used in docs.rs is updated
#![cfg_attr(finchers_inject_extern_prelude, feature(extern_prelude))]

//! WebSocket support for Finchers based on tungstenite.
//!
//! # Example
//!
//! ```
//! #[macro_use]
//! extern crate finchers;
//! extern crate finchers_tungstenite;
//! # extern crate futures;
//!
//! use finchers::prelude::*;
//! use finchers_tungstenite::{
//!   Ws,
//!   WsTransport,
//! };
//!
//! # fn main() {
//! let endpoint = path!(@get / "ws" /)
//!     .and(finchers_tungstenite::ws())
//!     .map(|ws: Ws| {
//!         ws.on_upgrade(|ws: WsTransport| {
//!             // ...
//! #           drop(ws);
//! #           futures::future::ok(())
//!         })
//!     });
//! # drop(|| {
//! # finchers::server::start(endpoint)
//! #     .serve("127.0.0.1:4000")
//! #     .unwrap();
//! # });
//! # }
//! ```

#![doc(html_root_url = "https://docs.rs/finchers-tungstenite/0.2.0")]
#![warn(
    missing_docs,
    missing_debug_implementations,
    nonstandard_style,
    rust_2018_idioms,
    unused,
)]
//#![warn(rust_2018_compatibility)]
#![cfg_attr(test, deny(warnings))]
#![cfg_attr(test, doc(test(attr(deny(warnings)))))]

extern crate base64;
#[macro_use]
extern crate failure;
extern crate finchers;
extern crate futures;
extern crate http;
extern crate sha1;
extern crate tokio_tungstenite;

pub extern crate tungstenite;

mod handshake;

// re-exports
pub use self::handshake::{HandshakeError, HandshakeErrorKind};
pub use self::imp::{ws, Ws, WsEndpoint, WsTransport};

#[doc(no_inline)]
pub use tungstenite::error::Error as WsError;
#[doc(no_inline)]
pub use tungstenite::protocol::{Message, WebSocketConfig};

mod imp {
    use finchers;
    use finchers::endpoint::{ApplyContext, ApplyResult, Endpoint};
    use finchers::endpoints::upgrade::{Builder, UpgradedIo};
    use finchers::output::Output;

    use tungstenite::protocol::{Role, WebSocketConfig};

    use futures::{Async, Future, Poll};
    use http::header;
    use tokio_tungstenite::WebSocketStream;

    use handshake::{handshake, Accept};

    #[allow(missing_docs)]
    pub type WsTransport = WebSocketStream<UpgradedIo>;

    /// Create an endpoint which handles the WebSocket handshake request.
    pub fn ws() -> WsEndpoint {
        (WsEndpoint { _priv: () }).with_output::<(Ws,)>()
    }

    /// An instance of `Endpoint` which handles the WebSocket handshake request.
    #[derive(Debug, Copy, Clone)]
    pub struct WsEndpoint {
        _priv: (),
    }

    impl<'a> Endpoint<'a> for WsEndpoint {
        type Output = (Ws,);
        type Future = WsFuture;

        fn apply(&'a self, _: &mut ApplyContext<'_>) -> ApplyResult<Self::Future> {
            Ok(WsFuture { _priv: () })
        }
    }

    #[derive(Debug)]
    pub struct WsFuture {
        _priv: (),
    }

    impl Future for WsFuture {
        type Item = (Ws,);
        type Error = finchers::error::Error;

        fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
            let accept = finchers::endpoint::with_get_cx(|cx| handshake(cx))?;
            Ok(Async::Ready((Ws {
                builder: Builder::new(),
                accept,
                config: None,
            },)))
        }
    }

    /// A type representing the result of handshake handling.
    ///
    /// The value of this type is used to build a WebSocket process
    /// after upgrading the protocol.
    #[derive(Debug)]
    pub struct Ws {
        builder: Builder,
        accept: Accept,
        config: Option<WebSocketConfig>,
    }

    impl Ws {
        /// Sets the configuration for upgraded WebSocket connection.
        pub fn config(self, config: WebSocketConfig) -> Ws {
            Ws {
                config: Some(config),
                ..self
            }
        }

        /// Creates an `Output` with the specified function which constructs
        /// a `Future` representing the task after upgrading the protocol to
        /// WebSocket.
        pub fn on_upgrade<F, R>(self, upgrade: F) -> impl Output
        where
            F: FnOnce(WsTransport) -> R + Send + 'static,
            R: Future<Item = (), Error = ()> + Send + 'static,
        {
            let Self {
                builder,
                accept,
                config,
            } = self;

            builder
                .header(header::CONNECTION, "upgrade")
                .header(header::UPGRADE, "websocket")
                .header(header::SEC_WEBSOCKET_ACCEPT, &*accept.hash)
                .finish(move |upgraded| {
                    let ws_stream =
                        WebSocketStream::from_raw_socket(upgraded, Role::Server, config);
                    upgrade(ws_stream)
                })
        }
    }
}
