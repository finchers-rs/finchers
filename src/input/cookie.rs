//! Components for managing Cookie values.

// re-exports
pub use cookie::{Cookie, CookieBuilder, EncodedCookie, SameSite};
#[cfg(feature = "secure")]
pub use cookie::{Key, PrivateJar, SignedJar};
pub use time::{Duration, Tm};

use cookie::CookieJar;
use failure::Error;
use http::header::HeaderMap;
use std::marker::PhantomData;
use std::rc::Rc;

/// A proxy object for tracking changes of Cookie values.
#[derive(Debug)]
pub struct Cookies<'a> {
    pub(super) jar: &'a mut CookieJar,
    pub(super) _marker: PhantomData<Rc<()>>,
}

impl<'a> Cookies<'a> {
    /// Returns a reference to the `Cookie` with the specified name.
    #[inline]
    pub fn get(&self, name: &str) -> Option<&Cookie<'static>> {
        self.jar.get(name)
    }

    /// Adds a `Cookie` to this jar.
    #[inline]
    pub fn add(&mut self, cookie: Cookie<'static>) {
        self.jar.add(cookie)
    }

    /// Removes a `Cookie` from this jar.
    #[inline]
    pub fn remove(&mut self, cookie: Cookie<'static>) {
        self.jar.remove(cookie)
    }

    /// Removes a `Cookie` from this jar completely.
    ///
    /// See [the documentation of `cookie`][force-remove] for details.
    ///
    /// [force-remove]:
    /// https://docs.rs/cookie/0.11/cookie/struct.CookieJar.html#method.force_remove
    #[inline]
    pub fn force_remove(&mut self, cookie: Cookie<'static>) {
        self.jar.force_remove(cookie)
    }

    /// Returns a `SignedJar` with `self`.
    #[cfg(feature = "secure")]
    #[inline]
    pub fn signed<'c>(&'c mut self, key: &Key) -> SignedJar<'c> {
        self.jar.signed(key)
    }

    /// Returns a `PrivateJar` with `self`.
    #[cfg(feature = "secure")]
    #[inline]
    pub fn private<'c>(&'c mut self, key: &Key) -> PrivateJar<'c> {
        self.jar.private(key)
    }
}

pub(super) fn parse_cookies(h: &HeaderMap) -> Result<CookieJar, Error> {
    let mut cookie_jar = CookieJar::new();

    for cookie in h.get_all(http::header::COOKIE) {
        let cookie_str = cookie.to_str()?;
        for s in cookie_str.split(';').map(|s| s.trim()) {
            let cookie = Cookie::parse_encoded(s)?.into_owned();
            cookie_jar.add_original(cookie);
        }
    }

    Ok(cookie_jar)
}
