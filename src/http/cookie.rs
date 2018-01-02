#![allow(missing_docs)]

pub use cookie::Cookie;

use std::fmt;
use std::ops::Deref;
use cookie::{CookieJar, Key};
use super::header;

pub struct Cookies {
    inner: CookieJar,
    key: SecretKey,
}

impl fmt::Debug for Cookies {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Cookies")
            .field("inner", &self.inner)
            .field("key", &"[secret]")
            .finish()
    }
}

impl Cookies {
    pub fn from_original(cookies: Option<&header::Cookie>, key: SecretKey) -> Self {
        let mut inner = CookieJar::new();
        if let Some(cookies) = cookies {
            for (name, value) in cookies.iter() {
                inner.add_original(Cookie::new(name.to_owned(), value.to_owned()));
            }
        }
        Cookies { inner, key }
    }

    pub fn get(&self, name: &str) -> Option<&Cookie<'static>> {
        self.inner.get(name)
    }

    pub fn get_private(&mut self, name: &str) -> Option<Cookie<'static>> {
        self.inner.private(&self.key).get(name)
    }

    pub fn add(&mut self, cookie: Cookie<'static>) {
        self.inner.add(cookie)
    }

    pub fn add_private(&mut self, cookie: Cookie<'static>) {
        self.inner.private(&self.key).add(cookie)
    }

    pub fn remove(&mut self, cookie: Cookie<'static>) {
        self.inner.remove(cookie)
    }

    pub fn remove_private(&mut self, cookie: Cookie<'static>) {
        self.inner.private(&self.key).remove(cookie)
    }

    pub fn collect_changes(&self) -> Vec<String> {
        self.inner
            .delta()
            .map(|c| c.encoded().to_string())
            .collect()
    }
}

#[derive(Clone)]
pub enum SecretKey {
    Generated(Key),
    Provided(Key),
}

impl fmt::Debug for SecretKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = match *self {
            SecretKey::Generated(..) => "Generated",
            SecretKey::Provided(..) => "Provided",
        };
        f.debug_tuple(name).field(&"[secret]").finish()
    }
}

impl Deref for SecretKey {
    type Target = Key;

    fn deref(&self) -> &Self::Target {
        match *self {
            SecretKey::Generated(ref key) | SecretKey::Provided(ref key) => key,
        }
    }
}

impl SecretKey {
    pub fn generated() -> Self {
        SecretKey::Generated(Key::generate())
    }

    pub fn provided<K: AsRef<[u8]>>(key: K) -> Self {
        SecretKey::Provided(Key::from_master(key.as_ref()))
    }
}
