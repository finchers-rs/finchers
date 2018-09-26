//! Endpoints for managing Cookie values.

use std::borrow::Cow;
use std::fmt;

use cookie::Cookie;
#[cfg(feature = "secure")]
use cookie::Key;

use endpoint::{Context, Endpoint, EndpointResult};
use error::{bad_request, Error};
use input::Input;

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
        input: &mut Input,
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
/// # #[macro_use]
/// # extern crate finchers;
/// # use finchers::endpoints::cookie;
/// # use finchers::prelude::*;
/// #
/// # fn main() {
/// let home = path!(@get / "home")
///     .and(routes![
///         cookie::required("session")
///             .map(|_| "authorized"),
///         endpoint::unit().map(|| "unauthorized"),
///     ]);
/// # drop(home);
/// # }
/// ```
#[inline]
pub fn required(name: impl Into<Cow<'static, str>>) -> Required {
    (Required {
        name: name.into(),
        mode: Mode::Plain,
    }).with_output::<(Cookie<'static>,)>()
}

#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct Required {
    name: Cow<'static, str>,
    mode: Mode,
}

impl Required {
    #[cfg(feature = "secure")]
    #[allow(missing_docs)]
    pub fn signed(self, key: Key) -> Required {
        Required {
            mode: Mode::Signed(key),
            ..self
        }
    }

    #[cfg(feature = "secure")]
    #[allow(missing_docs)]
    pub fn private(self, key: Key) -> Required {
        Required {
            mode: Mode::Private(key),
            ..self
        }
    }
}

impl<'a> Endpoint<'a> for Required {
    type Output = (Cookie<'static>,);
    type Future = ::futures::future::FutureResult<Self::Output, Error>;

    fn apply(&self, ecx: &mut Context<'_>) -> EndpointResult<Self::Future> {
        let cookie = self
            .mode
            .extract_cookie(ecx.input(), &self.name)
            .and_then(|cookie| {
                cookie.ok_or_else(|| bad_request(format!("missing Cookie item: {}", self.name)))
            });
        Ok(::futures::future::result(cookie.map(|x| (x,))))
    }
}

/// Create an endpoint which extracts a Cookie value.
///
/// This endpoint always accepts the request and will return a `None` if the value is missing.
///
/// # Example
///
/// ```
/// # #[macro_use]
/// # extern crate finchers;
/// # extern crate cookie;
/// # use finchers::endpoints::cookie::optional;
/// # use finchers::prelude::*;
/// # use cookie::Cookie;
/// #
/// # fn main() {
/// let home = path!(@get / "home")
///     .and(optional("session"))
///     .map(|c: Option<Cookie>| {
///         // ...
/// #       drop(c);
/// #       ()
///     });
/// # drop(home);
/// # }
/// ```
#[inline]
pub fn optional(name: impl Into<Cow<'static, str>>) -> Optional {
    (Optional {
        name: name.into(),
        mode: Mode::Plain,
    }).with_output::<(Option<Cookie<'static>>,)>()
}

#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct Optional {
    name: Cow<'static, str>,
    mode: Mode,
}

impl Optional {
    #[cfg(feature = "secure")]
    #[allow(missing_docs)]
    pub fn signed(self, key: Key) -> Optional {
        Optional {
            mode: Mode::Signed(key),
            ..self
        }
    }

    #[cfg(feature = "secure")]
    #[allow(missing_docs)]
    pub fn private(self, key: Key) -> Optional {
        Optional {
            mode: Mode::Private(key),
            ..self
        }
    }
}

impl<'a> Endpoint<'a> for Optional {
    type Output = (Option<Cookie<'static>>,);
    type Future = ::futures::future::FutureResult<Self::Output, Error>;

    fn apply(&self, ecx: &mut Context<'_>) -> EndpointResult<Self::Future> {
        Ok(::futures::future::result(
            self.mode
                .extract_cookie(ecx.input(), &self.name)
                .map(|x| (x,)),
        ))
    }
}
