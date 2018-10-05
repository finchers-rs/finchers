use hyper::body::Body;

/// A type holding the instance of request body.
#[derive(Debug)]
pub struct ReqBody(Option<Body>);

impl ReqBody {
    /// Create an instance of `RequestBody` from `hyper::Body`.
    #[deprecated(
        since = "0.12.3",
        note = "This method will be removed in the future version."
    )]
    #[inline]
    pub fn from_hyp(body: Body) -> ReqBody {
        ReqBody::new(body)
    }

    pub(crate) fn new(body: Body) -> ReqBody {
        ReqBody(Some(body))
    }

    #[allow(missing_docs)]
    pub fn payload(&mut self) -> Option<Body> {
        self.0.take()
    }

    #[allow(missing_docs)]
    pub fn is_gone(&self) -> bool {
        self.0.is_none()
    }

    #[cfg(feature = "rt")]
    pub(crate) fn content_length(&self) -> Option<u64> {
        use hyper::body::Payload;
        self.0.as_ref()?.content_length()
    }
}
