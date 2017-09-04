use hyper;
use super::{FromBody, ParseBody};


/// The instance of request body.
#[derive(Default, Debug)]
pub struct Body {
    inner: hyper::Body,
}

impl From<hyper::Body> for Body {
    fn from(body: hyper::Body) -> Self {
        Self { inner: body }
    }
}

impl<T: FromBody> Into<ParseBody<T>> for Body {
    fn into(self) -> ParseBody<T> {
        ParseBody::new(self.inner)
    }
}
