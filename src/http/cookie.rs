#![allow(missing_docs)]

pub use cookie::Cookie;

use cookie::CookieJar;
use super::{header, Request};

#[derive(Debug)]
pub struct Cookies {
    inner: CookieJar,
}

impl Cookies {
    pub fn get(&self, name: &str) -> Option<&Cookie<'static>> {
        self.inner.get(name)
    }

    pub fn add(&mut self, cookie: Cookie<'static>) {
        self.inner.add(cookie)
    }

    pub fn remove(&mut self, cookie: Cookie<'static>) {
        self.inner.remove(cookie)
    }

    pub fn collect_changes(&self) -> Vec<String> {
        self.inner.delta().map(|c| c.to_string()).collect()
    }
}

#[derive(Debug, Default, Clone)]
pub struct CookieManager {}

impl CookieManager {
    // TODO: support for generation private/signed key

    pub fn new_cookies(&self, request: &Request) -> Cookies {
        let mut inner = CookieJar::new();
        if let Some(cookies) = request.header::<header::Cookie>() {
            for (name, value) in cookies.iter() {
                inner.add_original(Cookie::new(name.to_owned(), value.to_owned()));
            }
        }
        Cookies { inner }
    }
}
