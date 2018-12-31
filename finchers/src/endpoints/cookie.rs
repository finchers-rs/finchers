//! Endpoints for managing Cookie values.

use std::borrow::Cow;
use std::fmt;

use cookie::Cookie;
#[cfg(feature = "secure")]
use cookie::Key;
use futures::future;

use crate::endpoint::{ApplyContext, ApplyResult, Endpoint};
use crate::error::{bad_request, Error};
use crate::input::{Cookies, Input};

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
    #[allow(deprecated)]
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

#[doc(hidden)]
#[deprecated(since = "0.13.5", note = "use `endpoints::cookies()` instead.")]
#[allow(deprecated)]
#[inline]
pub fn required(name: impl Into<Cow<'static, str>>) -> Required {
    (Required {
        name: name.into(),
        mode: Mode::Plain,
    })
    .with_output::<(Cookie<'static>,)>()
}

#[doc(hidden)]
#[deprecated(since = "0.13.5", note = "use `endpoints::cookies()` instead.")]
pub struct Required {
    name: Cow<'static, str>,
    mode: Mode,
}

#[allow(deprecated)]
impl fmt::Debug for Required {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Required")
            .field("name", &self.name)
            .field("mode", &self.mode)
            .finish()
    }
}

#[allow(deprecated)]
impl Clone for Required {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            mode: self.mode.clone(),
        }
    }
}

#[allow(deprecated)]
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

#[allow(deprecated)]
impl Endpoint for Required {
    type Output = (Cookie<'static>,);
    type Future = future::FutureResult<Self::Output, Error>;

    fn apply(&self, ecx: &mut ApplyContext<'_>) -> ApplyResult<Self::Future> {
        let cookie = self
            .mode
            .extract_cookie(ecx.input(), &self.name)
            .and_then(|cookie| {
                cookie.ok_or_else(|| bad_request(format!("missing Cookie item: {}", self.name)))
            });
        Ok(future::result(cookie.map(|x| (x,))))
    }
}

#[doc(hidden)]
#[deprecated(since = "0.13.5", note = "use `endpoints::cookies()` instead.")]
#[allow(deprecated)]
#[inline]
pub fn optional(name: impl Into<Cow<'static, str>>) -> Optional {
    (Optional {
        name: name.into(),
        mode: Mode::Plain,
    })
    .with_output::<(Option<Cookie<'static>>,)>()
}

#[doc(hidden)]
#[deprecated(since = "0.13.5", note = "use `endpoints::cookies()` instead.")]
pub struct Optional {
    name: Cow<'static, str>,
    mode: Mode,
}

#[allow(deprecated)]
impl fmt::Debug for Optional {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Optional")
            .field("name", &self.name)
            .field("mode", &self.mode)
            .finish()
    }
}

#[allow(deprecated)]
impl Clone for Optional {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            mode: self.mode.clone(),
        }
    }
}

#[allow(deprecated)]
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

#[allow(deprecated)]
impl Endpoint for Optional {
    type Output = (Option<Cookie<'static>>,);
    type Future = future::FutureResult<Self::Output, Error>;

    fn apply(&self, ecx: &mut ApplyContext<'_>) -> ApplyResult<Self::Future> {
        Ok(future::result(
            self.mode
                .extract_cookie(ecx.input(), &self.name)
                .map(|x| (x,)),
        ))
    }
}

// ==== cookies ====

/// Creates an endpoint which returns an object for tracking Cookie values.
///
/// # Example
///
/// ```
/// # extern crate finchers;
/// # use finchers::prelude::*;
/// # use finchers::endpoints::cookie::cookies;
/// # use finchers::input::Cookies;
/// #
/// # fn main() {
/// let endpoint = cookies()
///     .map(|cookies: Cookies| {
///         let session_id = cookies.get("session-id");
///         // ...
/// #       drop(session_id);
/// #       ()
///     });
/// # drop(endpoint);
/// # }
/// ```
pub fn cookies() -> CookiesEndpoint {
    CookiesEndpoint { _priv: () }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct CookiesEndpoint {
    _priv: (),
}

impl Endpoint for CookiesEndpoint {
    type Output = (Cookies,);
    type Future = future::FutureResult<Self::Output, Error>;

    fn apply(&self, cx: &mut ApplyContext<'_>) -> ApplyResult<Self::Future> {
        Ok(future::result(
            cx.input().cookies2().map(|cookies| (cookies,)),
        ))
    }
}
