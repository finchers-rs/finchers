use http::header::HeaderMap;
use http::header::COOKIE;
use std::cell::UnsafeCell;
use std::marker::PhantomData;

use cookie::Cookie;
pub(super) use cookie::CookieJar;
#[cfg(feature = "secure")]
use cookie::Key;

use crate::endpoint::with_get_cx;
use crate::error;
use crate::error::Error;

/// A proxy object for tracking Cookie values in the current context.
#[derive(Debug)]
pub struct Cookies {
    _marker: PhantomData<UnsafeCell<()>>,
}

impl Cookies {
    fn with_get_jar<R>(f: impl FnOnce(&mut CookieJar) -> R) -> R {
        with_get_cx(|cx| {
            let jar = cx
                .input()
                .cookie_manager
                .jar()
                .expect("should be available");
            f(jar)
        })
    }

    /// Return a Cookie with the specified name from the jar in the current context.
    pub fn get(&self, name: &str) -> Option<Cookie<'static>> {
        Cookies::with_get_jar(|jar| jar.get(name).cloned())
    }

    /// Adds a Cookie entry to the jar in the current context.
    pub fn add(&mut self, cookie: Cookie<'static>) {
        Cookies::with_get_jar(|jar| jar.add(cookie))
    }

    /// Removes a Cookie entry from the jar in the current context.
    pub fn remove(&mut self, cookie: Cookie<'static>) {
        Cookies::with_get_jar(|jar| jar.remove(cookie))
    }

    /// Removes *completely* a Cookie entry from the jar in the current context.
    pub fn force_remove(&mut self, cookie: Cookie<'static>) {
        Cookies::with_get_jar(|jar| jar.force_remove(cookie))
    }
}

#[allow(missing_docs)]
#[cfg(feature = "secure")]
impl Cookies {
    pub fn get_signed(&self, key: &Key, name: &str) -> Option<Cookie<'static>> {
        Cookies::with_get_jar(|jar| jar.signed(key).get(name))
    }

    pub fn get_private(&self, key: &Key, name: &str) -> Option<Cookie<'static>> {
        Cookies::with_get_jar(|jar| jar.private(key).get(name))
    }

    pub fn add_signed(&mut self, key: &Key, cookie: Cookie<'static>) {
        Cookies::with_get_jar(|jar| jar.signed(key).add(cookie))
    }

    pub fn add_private(&mut self, key: &Key, cookie: Cookie<'static>) {
        Cookies::with_get_jar(|jar| jar.private(key).add(cookie))
    }

    pub fn remove_signed(&mut self, key: &Key, cookie: Cookie<'static>) {
        Cookies::with_get_jar(|jar| jar.signed(key).remove(cookie))
    }

    pub fn remove_private(&mut self, key: &Key, cookie: Cookie<'static>) {
        Cookies::with_get_jar(|jar| jar.private(key).remove(cookie))
    }
}

#[derive(Debug, Default)]
pub(super) struct CookieManager {
    jar: Option<CookieJar>,
}

impl CookieManager {
    pub(super) fn ensure_initialized(&mut self, headers: &HeaderMap) -> Result<Cookies, Error> {
        if self.jar.is_none() {
            let mut cookie_jar = CookieJar::new();
            for cookie in headers.get_all(COOKIE) {
                let cookie_str = cookie.to_str().map_err(error::bad_request)?;
                for s in cookie_str.split(';').map(|s| s.trim()) {
                    let cookie = Cookie::parse_encoded(s)
                        .map_err(error::bad_request)?
                        .into_owned();
                    cookie_jar.add_original(cookie);
                }
            }
            self.jar.get_or_insert(cookie_jar);
        }

        Ok(Cookies {
            _marker: PhantomData,
        })
    }

    pub(super) fn jar(&mut self) -> Option<&mut CookieJar> {
        self.jar.as_mut()
    }

    pub(super) fn into_inner(self) -> Option<CookieJar> {
        self.jar
    }
}
