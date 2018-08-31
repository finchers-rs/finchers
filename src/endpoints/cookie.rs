//! Endpoints for managing Cookie values.

use std::borrow::Cow;
use std::fmt;
use std::pin::PinMut;

use futures_util::future::{ready, Ready};

use crate::endpoint::{Context, Endpoint, EndpointResult};
use crate::error::{bad_request, Error};
use crate::input::cookie::Cookie;
#[cfg(feature = "secure")]
use crate::input::cookie::Key;
use crate::input::Input;

#[derive(Clone)]
enum Mode {
    Plain,
    #[cfg(feature = "secure")]
    Signed(Key),
    #[cfg(feature = "secure")]
    Private(Key),
}

impl fmt::Debug for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Mode").finish()
    }
}

impl Mode {
    fn extract_cookie(
        &self,
        input: PinMut<'_, Input>,
        name: &str,
    ) -> Result<Option<Cookie<'static>>, Error> {
        let cookies = input.cookies()?;
        match self {
            Mode::Plain => Ok(cookies.get(name).cloned()),
            #[cfg(feature = "secure")]
            Mode::Signed(ref key) => Ok(cookies.signed(key).get(name)),
            #[cfg(feature = "secure")]
            Mode::Private(ref key) => Ok(cookies.private(key).get(name)),
        }
    }
}

/// Create an endpoint which extracts a Cookie value.
///
/// If the value is not found, it will skip the current request.
///
/// # Example
///
/// ```
/// # use finchers::endpoints::cookie;
/// # use finchers::endpoint::{unit, EndpointExt};
/// # use finchers::{route, routes};
/// #
/// let home = route!(@get / "home")
///     .and(routes![
///         cookie::required("session")
///             .map(|_| "authorized"),
///         unit().map(|| "unauthorized"),
///     ]);
/// # drop(home);
/// ```
pub fn required(name: impl Into<Cow<'static, str>>) -> Required {
    Required {
        name: name.into(),
        mode: Mode::Plain,
    }
}

#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct Required {
    name: Cow<'static, str>,
    mode: Mode,
}

impl Required {
    #[cfg(feature = "secure")]
    pub fn signed(self, key: Key) -> Required {
        Required {
            mode: Mode::Signed(key),
            ..self
        }
    }

    #[cfg(feature = "secure")]
    pub fn private(self, key: Key) -> Required {
        Required {
            mode: Mode::Private(key),
            ..self
        }
    }
}

impl<'a> Endpoint<'a> for Required {
    type Output = (Cookie<'static>,);
    type Future = Ready<Result<Self::Output, Error>>;

    fn apply(&self, ecx: &mut Context<'_>) -> EndpointResult<Self::Future> {
        let cookie = self
            .mode
            .extract_cookie(ecx.input(), &self.name)
            .and_then(|cookie| {
                cookie.ok_or_else(|| bad_request(format!("missing Cookie item: {}", self.name)))
            });
        Ok(ready(cookie.map(|x| (x,))))
    }
}

/// Create an endpoint which extracts a Cookie value.
///
/// This endpoint always accepts the request and will return a `None` if the value is missing.
///
/// # Example
///
/// ```
/// # use finchers::endpoints::cookie;
/// # use finchers::endpoint::EndpointExt;
/// # use finchers::input::cookie::Cookie;
/// # use finchers::route;
/// #
/// let home = route!(@get / "home")
///     .and(cookie::optional("session"))
///     .map(|c: Option<Cookie>| {
///         // ...
/// #       drop(c);
/// #       ()
///     });
/// # drop(home);
/// ```
pub fn optional(name: impl Into<Cow<'static, str>>) -> Optional {
    Optional {
        name: name.into(),
        mode: Mode::Plain,
    }
}

#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct Optional {
    name: Cow<'static, str>,
    mode: Mode,
}

impl Optional {
    #[cfg(feature = "secure")]
    pub fn signed(self, key: Key) -> Optional {
        Optional {
            mode: Mode::Signed(key),
            ..self
        }
    }

    #[cfg(feature = "secure")]
    pub fn private(self, key: Key) -> Optional {
        Optional {
            mode: Mode::Private(key),
            ..self
        }
    }
}

impl<'a> Endpoint<'a> for Optional {
    type Output = (Option<Cookie<'static>>,);
    type Future = Ready<Result<Self::Output, Error>>;

    fn apply(&self, ecx: &mut Context<'_>) -> EndpointResult<Self::Future> {
        Ok(ready(
            self.mode
                .extract_cookie(ecx.input(), &self.name)
                .map(|x| (x,)),
        ))
    }
}
