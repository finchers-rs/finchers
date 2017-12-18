use hyper;


/// The instance of request body.
#[derive(Default, Debug)]
pub struct Body {
    pub(crate) inner: hyper::Body,
}

impl From<hyper::Body> for Body {
    fn from(body: hyper::Body) -> Self {
        Self { inner: body }
    }
}
