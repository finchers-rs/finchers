use failure::Error;
use std::error;
use std::fmt;

/// A type alias of `Result<T, E>` whose error type is restrected to `ServerError`.
pub type ServerResult<T> = Result<T, ServerError>;

#[derive(Debug)]
enum ServerErrorKind {
    Config(Error),
    Custom(Error),
}

/// The error type which will be returned from `ServiceBuilder::serve()`.
#[derive(Debug)]
pub struct ServerError {
    kind: ServerErrorKind,
}

impl ServerError {
    pub(super) fn config(err: impl Into<Error>) -> ServerError {
        ServerError {
            kind: ServerErrorKind::Config(err.into()),
        }
    }

    /// Create a value of `ServerError` from an arbitrary error value.
    pub fn custom(err: impl Into<Error>) -> ServerError {
        ServerError {
            kind: ServerErrorKind::Custom(err.into()),
        }
    }
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::ServerErrorKind::*;
        match self.kind {
            Config(ref e) => write!(f, "failed to build server config: {}", e),
            Custom(ref e) => fmt::Display::fmt(e, f),
        }
    }
}

impl error::Error for ServerError {
    fn description(&self) -> &str {
        "failed to start the server"
    }
}
