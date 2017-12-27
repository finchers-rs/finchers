pub use cookie::{Cookie, CookieJar};
use super::{header, Request};

pub fn init_cookie_jar(request: &Request) -> CookieJar {
    let mut jar = CookieJar::new();
    if let Some(cookies) = request.header::<header::Cookie>() {
        for (name, value) in cookies.iter() {
            jar.add_original(Cookie::new(name.to_owned(), value.to_owned()));
        }
    }
    jar
}
