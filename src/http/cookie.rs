#![allow(missing_docs)]

pub use cookie::Cookie;

use std::fmt;
use std::sync::Arc;
use cookie::{CookieJar, Key};
use super::header;

pub struct Cookies {
    inner: CookieJar,
    key: Arc<Key>,
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
    pub fn get(&self, name: &str) -> Option<&Cookie<'static>> {
        self.inner.get(name)
    }

    pub fn get_private(&mut self, name: &str) -> Option<Cookie<'static>> {
        self.inner.private(&*self.key).get(name)
    }

    pub fn add(&mut self, cookie: Cookie<'static>) {
        self.inner.add(cookie)
    }

    pub fn add_private(&mut self, cookie: Cookie<'static>) {
        self.inner.private(&*self.key).add(cookie)
    }

    pub fn remove(&mut self, cookie: Cookie<'static>) {
        self.inner.remove(cookie)
    }

    pub fn remove_private(&mut self, cookie: Cookie<'static>) {
        self.inner.private(&*self.key).remove(cookie)
    }

    pub fn collect_changes(&self) -> Vec<String> {
        self.inner
            .delta()
            .map(|c| c.encoded().to_string())
            .collect()
    }
}

#[derive(Clone)]
pub struct CookieManager {
    secret_key: Arc<Key>,
}

impl fmt::Debug for CookieManager {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("CookieManager")
            .field("key", &"[secret]")
            .finish()
    }
}

impl Default for CookieManager {
    fn default() -> Self {
        CookieManager {
            secret_key: Arc::new(Key::generate()),
        }
    }
}

impl CookieManager {
    pub fn new<K: AsRef<[u8]>>(key: K) -> Self {
        CookieManager {
            secret_key: Arc::new(Key::from_master(key.as_ref())),
        }
    }

    pub fn set_secret_key<K: AsRef<[u8]>>(&mut self, key: K) {
        let key = Key::from_master(key.as_ref());
        self.secret_key = Arc::new(key);
    }

    pub fn generate_secret_key(&mut self) {
        self.secret_key = Arc::new(Key::generate());
    }

    pub fn new_cookies(&self, cookies: Option<&header::Cookie>) -> Cookies {
        let mut inner = CookieJar::new();
        if let Some(cookies) = cookies {
            for (name, value) in cookies.iter() {
                inner.add_original(Cookie::new(name.to_owned(), value.to_owned()));
            }
        }
        Cookies {
            inner,
            key: self.secret_key.clone(),
        }
    }
}
