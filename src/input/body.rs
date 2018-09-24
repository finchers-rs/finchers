use hyper::body::Body;

/// A type holding the instance of request body.
#[derive(Debug)]
pub struct ReqBody(Option<Body>);

impl ReqBody {
    /// Create an instance of `RequestBody` from `hyper::Body`.
    pub fn from_hyp(body: Body) -> ReqBody {
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
}
