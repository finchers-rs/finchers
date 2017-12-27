use http::{CookieJar, Request};

#[derive(Debug)]
pub struct ResponderContext {
    pub(crate) request: Request,
    pub(crate) cookies: CookieJar,
}
