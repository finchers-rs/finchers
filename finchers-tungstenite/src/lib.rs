//! WebSocket support for Finchers based on tungstenite.
//!
//! # Example
//!
//! ```
//! # use finchers::prelude::*;
//! type Transport = finchers_tungstenite::WsTransport<izanami::RequestBody>;
//!
//! # fn main() -> izanami::Result<()> {
//! let endpoint = finchers::path!(@get "/ws")
//!     .and(finchers_tungstenite::ws(|ws: Transport| {
//!             // ...
//! #           drop(ws);
//! #           futures::future::ok(())
//!     }));
//! # drop(|| {
//! # izanami::Server::build().start(endpoint.into_service())
//! # });
//! # Ok(())
//! # }
//! ```

#![doc(html_root_url = "https://docs.rs/finchers-tungstenite/0.3.0-dev")]
#![deny(
    missing_docs,
    missing_debug_implementations,
    nonstandard_style,
    rust_2018_compatibility,
    rust_2018_idioms,
    unused
)]
#![forbid(clippy::unimplemented)]
#![cfg_attr(test, doc(test(attr(deny(warnings)))))]

mod handshake;

#[doc(no_inline)]
pub use tungstenite::{
    self,
    error::Error as WsError,
    protocol::{Message, WebSocketConfig},
};

// re-exports
pub use crate::{
    handshake::{HandshakeError, HandshakeErrorKind},
    imp::{ws, WsEndpoint, WsTransport},
};

mod imp {
    use {
        crate::handshake::handshake,
        finchers::{
            action::{
                ActionContext, //
                EndpointAction,
                Preflight,
                PreflightContext,
            },
            endpoint::{Endpoint, IsEndpoint},
            error::Error,
        },
        futures::{Async, Future, IntoFuture, Poll},
        http::{header, Response},
        izanami_http::upgrade::Upgrade,
        izanami_rt::{DefaultExecutor, Executor},
        tokio_tungstenite::WebSocketStream,
        tungstenite::protocol::Role,
    };

    #[allow(missing_docs)]
    pub type WsTransport<Bd> = WebSocketStream<<Bd as Upgrade>::Upgraded>;

    /// Create an endpoint which handles the WebSocket handshake request.
    pub fn ws<F>(on_upgrade: F) -> WsEndpoint<F> {
        WsEndpoint { on_upgrade }
    }

    /// An instance of `Endpoint` which handles the WebSocket handshake request.
    #[derive(Debug, Copy, Clone)]
    pub struct WsEndpoint<F> {
        on_upgrade: F,
    }

    impl<F> IsEndpoint for WsEndpoint<F> {}

    impl<F, Bd, R> Endpoint<Bd> for WsEndpoint<F>
    where
        F: Fn(WsTransport<Bd>) -> R + Clone + Send + 'static,
        R: IntoFuture<Item = (), Error = ()>,
        R::Future: Send + 'static,
        Bd: Upgrade + Send + 'static,
        Bd::Upgraded: Send + 'static,
    {
        type Output = (Response<&'static str>,);
        type Action = WsAction<F>;

        fn action(&self) -> Self::Action {
            WsAction {
                on_upgrade: Some(self.on_upgrade.clone()),
            }
        }
    }

    #[derive(Debug)]
    pub struct WsAction<F> {
        on_upgrade: Option<F>,
    }

    impl<F, Bd, R> EndpointAction<Bd> for WsAction<F>
    where
        F: Fn(WsTransport<Bd>) -> R + Send + 'static,
        R: IntoFuture<Item = (), Error = ()>,
        R::Future: Send + 'static,
        Bd: Upgrade + Send + 'static,
        Bd::Upgraded: Send + 'static,
    {
        type Output = (Response<&'static str>,);

        fn preflight(
            &mut self,
            _: &mut PreflightContext<'_>,
        ) -> Result<Preflight<Self::Output>, Error> {
            Ok(Preflight::Incomplete)
        }

        fn poll_action(&mut self, cx: &mut ActionContext<'_, Bd>) -> Poll<Self::Output, Error> {
            let accept = handshake(cx)?;

            let on_upgrade = self
                .on_upgrade
                .take()
                .expect("the action has already been polled");
            let body = cx.body().take().ok_or_else(|| {
                finchers::error::InternalServerError::from(
                    "the instance of request body has already been stolen by someone.",
                )
            })?;
            let upgrade_task = body
                .on_upgrade()
                .map_err(|_e| log::error!("upgrade error"))
                .and_then(move |upgraded| {
                    let ws_stream = WebSocketStream::from_raw_socket(upgraded, Role::Server, None);
                    on_upgrade(ws_stream).into_future()
                });

            DefaultExecutor::current()
                .spawn(Box::new(upgrade_task))
                .map_err(finchers::error::InternalServerError::from)?;

            let response = Response::builder()
                .status(http::StatusCode::SWITCHING_PROTOCOLS)
                .header(header::CONNECTION, "upgrade")
                .header(header::UPGRADE, "websocket")
                .header(header::SEC_WEBSOCKET_ACCEPT, &*accept.hash)
                .body("")
                .unwrap();

            Ok(Async::Ready((response,)))
        }
    }
}
