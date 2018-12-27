//! The implementation of WebSocket handshake process.

use base64;
use http::header;
use http::StatusCode;
use sha1::Sha1;

use finchers::error::HttpError;
use finchers::input::Input;

#[derive(Debug)]
pub(crate) struct Accept {
    pub(crate) hash: String,
    _priv: (),
}

/// Check if the specified HTTP response is a valid WebSocket handshake request.
///
/// If successful, it returns a SHA1 hash used as `Sec-WebSocket-Accept` header in the response.
pub(crate) fn handshake(request: &Input) -> Result<Accept, HandshakeError> {
    let h = request
        .headers()
        .get(header::CONNECTION)
        .ok_or_else(|| HandshakeErrorKind::MissingHeader { name: "Connection" })?;
    if h != "Upgrade" && h != "upgrade" {
        return Err(HandshakeErrorKind::InvalidHeader { name: "Connection" }.into());
    }

    let h = request
        .headers()
        .get(header::UPGRADE)
        .ok_or_else(|| HandshakeErrorKind::MissingHeader { name: "Upgrade" })?;
    if h != "Websocket" && h != "websocket" {
        return Err(HandshakeErrorKind::InvalidHeader { name: "Upgrade" }.into());
    }

    let h = request
        .headers()
        .get(header::SEC_WEBSOCKET_VERSION)
        .ok_or_else(|| HandshakeErrorKind::MissingHeader {
            name: "Sec-WebSocket-Version",
        })?;
    if h != "13" {
        return Err(HandshakeErrorKind::InvalidSecWebSocketVersion.into());
    }

    let h = request
        .headers()
        .get(header::SEC_WEBSOCKET_KEY)
        .ok_or_else(|| HandshakeErrorKind::MissingHeader {
            name: "Sec-WebSocket-Key",
        })?;
    let decoded = base64::decode(h).map_err(|_| HandshakeErrorKind::InvalidSecWebSocketKey)?;
    if decoded.len() != 16 {
        return Err(HandshakeErrorKind::InvalidSecWebSocketKey.into());
    }

    let mut m = Sha1::new();
    m.update(h.as_bytes());
    m.update(b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11");

    let hash = base64::encode(&m.digest().bytes()[..]);

    Ok(Accept { hash, _priv: () })
}

/// The error type during handling WebSocket handshake.
#[derive(Debug, Fail)]
#[fail(display = "handshake error: {}", kind)]
pub struct HandshakeError {
    kind: HandshakeErrorKind,
}

impl From<HandshakeErrorKind> for HandshakeError {
    fn from(kind: HandshakeErrorKind) -> Self {
        HandshakeError { kind }
    }
}

impl HttpError for HandshakeError {
    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
}

impl HandshakeError {
    #[allow(missing_docs)]
    pub fn kind(&self) -> &HandshakeErrorKind {
        &self.kind
    }
}

#[allow(missing_docs)]
#[derive(Debug, Fail)]
#[cfg_attr(test, derive(PartialEq))]
pub enum HandshakeErrorKind {
    #[fail(display = "missing header: `{}'", name)]
    MissingHeader { name: &'static str },

    #[fail(display = "The header value is invalid: `{}'", name)]
    InvalidHeader { name: &'static str },

    #[fail(display = "The value of `Sec-WebSocket-Key` is invalid")]
    InvalidSecWebSocketKey,

    #[fail(display = "The value of `Sec-WebSocket-Version` must be equal to '13'")]
    InvalidSecWebSocketVersion,
}
