#![allow(missing_docs)]

pub use cookie::Cookie;

use std::fmt;
use std::ops::Deref;
use cookie::CookieJar;
use super::header;

#[cfg(feature = "secure")]
use cookie::Key;
#[cfg(not(feature = "secure"))]
#[derive(Debug, Clone)]
pub struct Key {
    _priv: (),
}

pub struct Cookies {
    inner: CookieJar,
    #[cfg_attr(not(feature = "secure"), allow(dead_code))] key: SecretKey,
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

    #[cfg(feature = "secure")]
    pub fn get_private(&mut self, name: &str) -> Option<Cookie<'static>> {
        self.inner.private(&self.key).get(name)
    }

    pub fn add(&mut self, cookie: Cookie<'static>) {
        self.inner.add(cookie)
    }

    #[cfg(feature = "secure")]
    pub fn add_private(&mut self, cookie: Cookie<'static>) {
        self.inner.private(&self.key).add(cookie)
    }

    pub fn remove(&mut self, cookie: Cookie<'static>) {
        self.inner.remove(cookie)
    }

    #[cfg(feature = "secure")]
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
    #[cfg(feature = "secure")]
    pub fn generated() -> Self {
        SecretKey::Generated(Key::generate())
    }

    #[cfg(not(feature = "secure"))]
    pub fn generated() -> Self {
        SecretKey::Generated(Key { _priv: () })
    }

    #[cfg(feature = "secure")]
    pub fn provided<K: AsRef<[u8]>>(key: K) -> Self {
        SecretKey::Provided(Key::from_master(key.as_ref()))
    }

    #[cfg(not(feature = "secure"))]
    pub fn provided<K: AsRef<[u8]>>(_key: K) -> Self {
        SecretKey::Provided(Key { _priv: () })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cookies_from_original() {
        let mut original = header::Cookie::new();
        original.set("SID", "31d4d96e407aad42");
        original.set("lang", "en-US");

        let cookies = Cookies::from_original(Some(original).as_ref(), SecretKey::generated());
        assert_eq!(
            cookies.get("SID"),
            Some(&Cookie::new("SID", "31d4d96e407aad42"))
        );
        assert_eq!(cookies.get("lang"), Some(&Cookie::new("lang", "en-US")));
    }
}
