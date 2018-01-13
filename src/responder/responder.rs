use std::borrow::Cow;
use http::{Headers, IntoBody, Response, StatusCode};

/// Abstrcution of types converted into a raw HTTP response.
pub trait Responder {
    /// The type of the value returned from `body`
    type Body: IntoBody;

    /// Returns the status code of the HTTP response
    ///
    /// The default value is `200 OK`.
    fn status(&self) -> StatusCode {
        StatusCode::Ok
    }

    /// Returns the instance of response body, if available.
    ///
    /// The default value is `None`.
    fn body(&mut self) -> Option<Self::Body> {
        None
    }

    /// Add additional headers to the response.
    ///
    /// By default, this method has no affect to the HTTP response.
    fn headers(&self, &mut Headers) {}

    #[allow(missing_docs)]
    fn respond<R: From<Response>>(&mut self) -> R {
        super::respond(self).into()
    }
}

impl Responder for () {
    type Body = ();

    fn status(&self) -> StatusCode {
        StatusCode::NoContent
    }
}

/// A responder with the body of string.
#[derive(Debug)]
pub struct StringResponder(Option<Cow<'static, str>>);

impl Responder for StringResponder {
    type Body = Cow<'static, str>;

    fn body(&mut self) -> Option<Self::Body> {
        self.0.take()
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct OptionResponder<R>(Option<R>);

impl<R: Responder> Responder for OptionResponder<R> {
    type Body = R::Body;

    fn status(&self) -> StatusCode {
        self.0.as_ref().map_or(StatusCode::NotFound, |r| r.status())
    }

    fn body(&mut self) -> Option<Self::Body> {
        self.0.as_mut().and_then(|r| r.body())
    }

    fn headers(&self, h: &mut Headers) {
        self.0.as_ref().map(|r| r.headers(h));
    }

    fn respond<T: From<Response>>(&mut self) -> T {
        if let Some(ref mut r) = self.0 {
            return r.respond();
        }
        super::respond(self).into()
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct ResultResponder<T, E>(Result<T, E>);

impl<T: Responder, E: Responder<Body = T::Body>> Responder for ResultResponder<T, E> {
    type Body = T::Body;

    fn status(&self) -> StatusCode {
        match self.0 {
            Ok(ref t) => t.status(),
            Err(ref e) => e.status(),
        }
    }

    fn body(&mut self) -> Option<Self::Body> {
        match self.0 {
            Ok(ref mut t) => t.body(),
            Err(ref mut e) => e.body(),
        }
    }

    fn headers(&self, h: &mut Headers) {
        match self.0 {
            Ok(ref t) => t.headers(h),
            Err(ref e) => e.headers(h),
        }
    }

    fn respond<R: From<Response>>(&mut self) -> R {
        match self.0 {
            Ok(ref mut t) => t.respond(),
            Err(ref mut e) => e.respond(),
        }
    }
}

/// Abstrcution of types to be convert to a `Responder`.
pub trait IntoResponder {
    /// The type of response body
    type Body: IntoBody;
    /// The type of returned value from `into_response`
    type Responder: Responder<Body = Self::Body>;

    /// Convert itself into `Self::Responder`
    fn into_responder(self) -> Self::Responder;
}

impl<R: Responder> IntoResponder for R {
    type Body = R::Body;
    type Responder = Self;

    fn into_responder(self) -> Self {
        self
    }
}

impl IntoResponder for &'static str {
    type Body = Cow<'static, str>;
    type Responder = StringResponder;

    fn into_responder(self) -> Self::Responder {
        StringResponder(Some(self.into()))
    }
}

impl IntoResponder for String {
    type Body = Cow<'static, str>;
    type Responder = StringResponder;

    fn into_responder(self) -> Self::Responder {
        StringResponder(Some(self.into()))
    }
}

impl IntoResponder for Cow<'static, str> {
    type Body = Cow<'static, str>;
    type Responder = StringResponder;

    fn into_responder(self) -> Self::Responder {
        StringResponder(Some(self))
    }
}

impl<R: IntoResponder> IntoResponder for Option<R> {
    type Body = R::Body;
    type Responder = OptionResponder<R::Responder>;

    fn into_responder(self) -> Self::Responder {
        OptionResponder(self.map(IntoResponder::into_responder))
    }
}

impl<T: IntoResponder, E: IntoResponder<Body = T::Body>> IntoResponder for Result<T, E> {
    type Body = T::Body;
    type Responder = ResultResponder<T::Responder, E::Responder>;

    fn into_responder(self) -> Self::Responder {
        ResultResponder(
            self.map(IntoResponder::into_responder)
                .map_err(IntoResponder::into_responder),
        )
    }
}
