use http::header::{HeaderValue, LOCATION};
use http::{Response, StatusCode};

use super::{Output, OutputContext};
use crate::error::Never;

/// An instance of `Output` representing redirect responses.
#[derive(Debug, Clone)]
pub struct Redirect {
    status: StatusCode,
    location: Option<HeaderValue>,
}

impl Redirect {
    /// Create a new `Redirect` with the specified HTTP status code.
    pub fn new(status: StatusCode) -> Redirect {
        Redirect {
            status,
            location: None,
        }
    }

    /// Sets the value of header field `Location`.
    pub fn location(self, location: &'static str) -> Redirect {
        Redirect {
            location: Some(HeaderValue::from_static(location)),
            ..self
        }
    }
}

macro_rules! impl_constructors {
    ($($name:ident => $STATUS:ident;)*) => {$(
        pub fn $name(location: &'static str) -> Redirect {
            Redirect {
                status: StatusCode::$STATUS,
                location: Some(HeaderValue::from_static(location)),
            }
        }
    )*}
}

#[allow(missing_docs)]
impl Redirect {
    impl_constructors! {
        moved_permanently => MOVED_PERMANENTLY;
        found => FOUND;
        see_other => SEE_OTHER;
        temporary_redirect => TEMPORARY_REDIRECT;
        permanent_redirect => PERMANENT_REDIRECT;
    }

    pub fn not_modified() -> Redirect {
        Redirect::new(StatusCode::NOT_MODIFIED)
    }
}

impl Output for Redirect {
    type Body = ();
    type Error = Never;

    fn respond(self, _: &mut OutputContext<'_>) -> Result<Response<Self::Body>, Self::Error> {
        let mut response = Response::new(());
        *response.status_mut() = self.status;
        if let Some(location) = self.location {
            response.headers_mut().insert(LOCATION, location);
        }
        Ok(response)
    }
}
