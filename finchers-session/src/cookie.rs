//! The session backend using Cookie as a session storage.
//!
//! # Example
//!
//! ```
//! #[macro_use]
//! extern crate finchers;
//! extern crate finchers_session;
//!
//! use finchers::prelude::*;
//! use finchers_session::Session;
//! use finchers_session::cookie::CookieSession;
//!
//! # fn main() {
//! let backend = finchers_session::cookie::plain();
//!
//! let endpoint = path!(@get /)
//!     .and(backend)
//!     .and_then(|session: Session<CookieSession>| {
//!         session.with(|_session| {
//!             // ...
//! #           Ok("done")
//!         })
//!     });
//! # drop(move || finchers::server::start(endpoint).serve("127.0.0.1:4000"));
//! # }
//! ```

extern crate cookie;

use finchers::endpoint::{ApplyContext, ApplyResult, Endpoint};
use finchers::error::Error;
use finchers::input::Input;

#[cfg(feature = "secure")]
use self::cookie::Key;
use self::cookie::{Cookie, SameSite};
use futures::future;
use std::borrow::Cow;
use std::fmt;
use std::sync::Arc;
use time::Duration;

use session::{RawSession, Session};
use util::BuilderExt;

// TODOs:
// * add support for setting whether to compress data

/// Create a `CookieSessionBackend` without signing and encryption.
///
/// This function is equivalent to `CookieSessionBackend::plain()`.
pub fn plain() -> CookieBackend {
    CookieBackend::plain()
}

/// Create a `CookieSessionBackend` with signing
/// (requires `feature = "secure"`).
///
/// This function is equivalent to `CookieSessionBackend::signed(Key::from_master(key.as_ref()))`.
#[cfg(feature = "secure")]
pub fn signed(master: impl AsRef<[u8]>) -> CookieBackend {
    CookieBackend::signed(Key::from_master(master.as_ref()))
}

/// Create a `CookieSessionBackend` with encryption
/// (requires `feature = "secure"`).
///
/// This function is equivalent to `CookieSessionBackend::private(Key::from_master(key.as_ref()))`.
#[cfg(feature = "secure")]
pub fn private(master: impl AsRef<[u8]>) -> CookieBackend {
    CookieBackend::private(Key::from_master(master.as_ref()))
}

enum Security {
    Plain,
    #[cfg(feature = "secure")]
    Signed(Key),
    #[cfg(feature = "secure")]
    Private(Key),
}

impl fmt::Debug for Security {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Security::Plain => f.debug_tuple("Plain").finish(),
            #[cfg(feature = "secure")]
            Security::Signed(..) => f.debug_tuple("Signed").finish(),
            #[cfg(feature = "secure")]
            Security::Private(..) => f.debug_tuple("Private").finish(),
        }
    }
}

#[derive(Debug)]
struct CookieConfig {
    security: Security,
    name: String,
    path: Cow<'static, str>,
    secure: bool,
    http_only: bool,
    domain: Option<Cow<'static, str>>,
    same_site: Option<SameSite>,
    max_age: Option<Duration>,
}

impl CookieConfig {
    fn read_value(&self, input: &mut Input) -> Result<Option<String>, Error> {
        let jar = input.cookies()?;
        let cookie = match self.security {
            Security::Plain => jar.get(&self.name).cloned(),
            #[cfg(feature = "secure")]
            Security::Signed(ref key) => jar.signed(key).get(&self.name),
            #[cfg(feature = "secure")]
            Security::Private(ref key) => jar.private(key).get(&self.name),
        };

        match cookie {
            Some(cookie) => Ok(Some(cookie.value().to_string())),
            None => Ok(None),
        }
    }

    fn write_value(&self, input: &mut Input, value: String) -> Result<(), Error> {
        let cookie = Cookie::build(self.name.clone(), value)
            .path(self.path.clone())
            .secure(self.secure)
            .http_only(self.http_only)
            .if_some(self.domain.clone(), |cookie, value| cookie.domain(value))
            .if_some(self.same_site, |cookie, value| cookie.same_site(value))
            .if_some(self.max_age, |cookie, value| cookie.max_age(value))
            .finish();

        let jar = input.cookies()?;
        match self.security {
            Security::Plain => jar.add(cookie),
            #[cfg(feature = "secure")]
            Security::Signed(ref key) => jar.signed(key).add(cookie),
            #[cfg(feature = "secure")]
            Security::Private(ref key) => jar.private(key).add(cookie),
        }

        Ok(())
    }

    fn remove_value(&self, input: &mut Input) -> Result<(), Error> {
        let cookie = Cookie::named(self.name.clone());
        let jar = input.cookies()?;
        match self.security {
            Security::Plain => jar.remove(cookie),
            #[cfg(feature = "secure")]
            Security::Signed(ref key) => jar.signed(key).remove(cookie),
            #[cfg(feature = "secure")]
            Security::Private(ref key) => jar.private(key).remove(cookie),
        }
        Ok(())
    }
}

#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct CookieBackend {
    config: Arc<CookieConfig>,
}

impl CookieBackend {
    fn new(security: Security) -> CookieBackend {
        CookieBackend {
            config: Arc::new(CookieConfig {
                security,
                name: "finchers-session".into(),
                path: "/".into(),
                domain: None,
                same_site: None,
                max_age: None,
                secure: true,
                http_only: true,
            }),
        }
    }

    /// Creates a `CookieSessionBackend` which stores the Cookie values as a raw form.
    pub fn plain() -> CookieBackend {
        CookieBackend::new(Security::Plain)
    }

    /// Creates a `CookieSessionBackend` which signs the Cookie values with the specified secret key.
    ///
    /// This method is only available if the feature flag `secure` is set.
    #[cfg(feature = "secure")]
    pub fn signed(key: Key) -> CookieBackend {
        CookieBackend::new(Security::Signed(key))
    }

    /// Creates a `CookieSessionBackend` which encrypts the Cookie values with the specified secret key.
    ///
    /// This method is only available if the feature flag `secure` is set.
    #[cfg(feature = "secure")]
    pub fn private(key: Key) -> CookieBackend {
        CookieBackend::new(Security::Private(key))
    }

    fn config_mut(&mut self) -> &mut CookieConfig {
        Arc::get_mut(&mut self.config).expect("The instance has already shared.")
    }

    /// Sets the path of Cookie entry.
    ///
    /// The default value is `"/"`.
    pub fn path(mut self, value: impl Into<Cow<'static, str>>) -> CookieBackend {
        self.config_mut().path = value.into();
        self
    }

    /// Sets the value of `secure` in Cookie entry.
    ///
    /// The default value is `true`.
    pub fn secure(mut self, value: bool) -> CookieBackend {
        self.config_mut().secure = value;
        self
    }

    /// Sets the value of `http_only` in Cookie entry.
    ///
    /// The default value is `true`.
    pub fn http_only(mut self, value: bool) -> CookieBackend {
        self.config_mut().http_only = value;
        self
    }

    /// Sets the value of `domain` in Cookie entry.
    ///
    /// The default value is `None`.
    pub fn domain(mut self, value: impl Into<Cow<'static, str>>) -> CookieBackend {
        self.config_mut().domain = Some(value.into());
        self
    }

    /// Sets the value of `same_site` in Cookie entry.
    ///
    /// The default value is `None`.
    pub fn same_site(mut self, value: SameSite) -> CookieBackend {
        self.config_mut().same_site = Some(value);
        self
    }

    /// Sets the value of `max_age` in Cookie entry.
    ///
    /// The default value is `None`.
    pub fn max_age(mut self, value: Duration) -> CookieBackend {
        self.config_mut().max_age = Some(value);
        self
    }
}

impl<'a> Endpoint<'a> for CookieBackend {
    type Output = (Session<CookieSession>,);
    type Future = future::FutureResult<Self::Output, Error>;

    fn apply(&self, cx: &mut ApplyContext<'_>) -> ApplyResult<Self::Future> {
        Ok(future::result(self.config.read_value(cx.input()).map(
            |value| {
                (Session::new(CookieSession {
                    config: self.config.clone(),
                    value,
                }),)
            },
        )))
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct CookieSession {
    config: Arc<CookieConfig>,
    value: Option<String>,
}

impl CookieSession {
    fn write_impl(self, input: &mut Input) -> Result<(), Error> {
        if let Some(value) = self.value {
            self.config.write_value(input, value)
        } else {
            self.config.remove_value(input)
        }
    }
}

impl RawSession for CookieSession {
    type WriteFuture = future::FutureResult<(), Error>;

    fn get(&self) -> Option<&str> {
        self.value.as_ref().map(|s| s.as_str())
    }

    fn set(&mut self, value: String) {
        self.value = Some(value);
    }

    fn remove(&mut self) {
        self.value = None;
    }

    fn write(self, input: &mut Input) -> Self::WriteFuture {
        future::result(self.write_impl(input))
    }
}
