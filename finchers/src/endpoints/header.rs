//! Components for parsing the HTTP headers.

use {
    crate::{
        action::{
            Oneshot,
            OneshotAction,
            PreflightContext, //
        },
        endpoint::{Endpoint, IsEndpoint},
        error::{BadRequest, Error},
        util::Never,
    },
    http::{
        header::{HeaderName, HeaderValue, ToStrError},
        HttpTryFrom,
    },
    mime::Mime,
    std::{fmt, marker::PhantomData},
    url::Url,
};

/// Trait representing the conversion from a header value.
pub trait FromHeaderValue: Sized + 'static {
    /// The error type which will be returned from `from_header_value()`.
    type Error: fmt::Debug + fmt::Display + Send + Sync + 'static;

    /// Perform conversion from a header value to `Self`.
    fn from_header_value(value: &HeaderValue) -> Result<Self, Self::Error>;
}

impl FromHeaderValue for HeaderValue {
    type Error = Never;

    fn from_header_value(value: &HeaderValue) -> Result<Self, Self::Error> {
        Ok(value.clone())
    }
}

impl FromHeaderValue for String {
    type Error = ToStrError;

    fn from_header_value(value: &HeaderValue) -> Result<Self, Self::Error> {
        value.to_str().map(ToOwned::to_owned)
    }
}

impl FromHeaderValue for Mime {
    type Error = failure::Error;

    fn from_header_value(value: &HeaderValue) -> Result<Self, Self::Error> {
        Ok(value.to_str()?.parse()?)
    }
}

impl FromHeaderValue for Url {
    type Error = failure::Error;

    fn from_header_value(value: &HeaderValue) -> Result<Self, Self::Error> {
        Ok(Url::parse(value.to_str()?)?)
    }
}

// ==== Parse ====

/// Create an endpoint which parses an entry in the HTTP header.
///
/// # Example
///
/// ```
/// # use finchers::endpoints::header;
/// let endpoint = header::parse::<String>("x-api-key");
/// # drop(endpoint);
/// ```
///
/// By default, this endpoint will skip the current request if the specified
/// name of header does not exist. In order to trait the missing header as an
/// error, use `or_reject_with` as follows:
///
/// ```ignore
/// # use finchers::prelude::*;
/// # use finchers::endpoints::header;
/// use finchers::endpoint::wrapper::or_reject_with;
/// use finchers::error::bad_request;
///
/// let endpoint = header::parse::<String>("x-api-key")
///     .wrap(or_reject_with(|_, _| bad_request("missing header: x-api-key")));
/// # drop(endpoint);
/// ```
#[inline]
pub fn parse<T>(name: &'static str) -> Parse<T>
where
    T: FromHeaderValue,
{
    Parse {
        name: HeaderName::from_static(name),
        _marker: PhantomData,
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct Parse<T> {
    name: HeaderName,
    _marker: PhantomData<fn() -> T>,
}

mod parse {
    use super::*;

    impl<T: FromHeaderValue> IsEndpoint for Parse<T> {}

    impl<T, Bd> Endpoint<Bd> for Parse<T>
    where
        T: FromHeaderValue,
    {
        type Output = (T,);
        type Action = Oneshot<ParseAction<T>>;

        fn action(&self) -> Self::Action {
            ParseAction {
                name: self.name.clone(),
                _marker: PhantomData,
            }
            .into_action()
        }
    }

    #[allow(missing_debug_implementations)]
    pub struct ParseAction<T> {
        name: HeaderName,
        _marker: PhantomData<fn() -> T>,
    }

    impl<T> OneshotAction for ParseAction<T>
    where
        T: FromHeaderValue,
    {
        type Output = (T,);

        fn preflight(self, cx: &mut PreflightContext<'_>) -> Result<Self::Output, Error> {
            let h = cx.headers().get(&self.name).ok_or_else(|| {
                BadRequest::from(format!("missing header: `{}'", self.name.as_str()))
            })?;
            T::from_header_value(h)
                .map(|parsed| (parsed,))
                .map_err(BadRequest::from)
                .map_err(Into::into)
        }
    }
}

// ==== Optional ====

/// Create an endpoint which parses an entry in the HTTP header.
///
/// This endpoint always matches to the request and it will return a `None`
/// if the value of specified header name is missing.
///
/// # Example
///
/// ```
/// # use finchers::endpoints::header;
/// let endpoint = header::optional::<String>("x-api-key");
/// # drop(endpoint);
/// ```
#[inline]
pub fn optional<T>(name: &'static str) -> Optional<T>
where
    T: FromHeaderValue,
{
    Optional {
        name: HeaderName::from_static(name),
        _marker: PhantomData,
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct Optional<T> {
    name: HeaderName,
    _marker: PhantomData<fn() -> T>,
}

mod optional {
    use super::*;

    impl<T: FromHeaderValue> IsEndpoint for Optional<T> {}

    impl<T, Bd> Endpoint<Bd> for Optional<T>
    where
        T: FromHeaderValue,
    {
        type Output = (Option<T>,);
        type Action = Oneshot<OptionalAction<T>>;

        fn action(&self) -> Self::Action {
            OptionalAction {
                name: self.name.clone(),
                _marker: PhantomData,
            }
            .into_action()
        }
    }

    #[allow(missing_debug_implementations)]
    pub struct OptionalAction<T> {
        name: HeaderName,
        _marker: PhantomData<fn() -> T>,
    }

    impl<T> OneshotAction for OptionalAction<T>
    where
        T: FromHeaderValue,
    {
        type Output = (Option<T>,);

        fn preflight(self, cx: &mut PreflightContext<'_>) -> Result<Self::Output, Error> {
            match cx.headers().get(&self.name) {
                Some(h) => T::from_header_value(h)
                    .map(|parsed| (Some(parsed),))
                    .map_err(BadRequest::from)
                    .map_err(Into::into),
                None => Ok((None,)),
            }
        }
    }
}

// ==== Matches ====

/// Create an endpoint which checks if a header entry with the specified name
/// is equal to the specified value.
///
/// If the header with the given name does not exists or it is not equal to
/// the specified value, it treats the request as not matching.
///
/// # Examples
///
/// ```
/// # use finchers::endpoints::header;
/// let endpoint = header::matches("origin", "www.example.com");
/// # drop(endpoint);
/// ```
///
/// ```ignore
/// # use finchers::prelude::*;
/// # use finchers::endpoints::header;
/// use finchers::error;
///
/// let endpoint = header::matches("origin", "www.example.com")
///     .wrap(endpoint::wrapper::or_reject_with(|_, _| error::bad_request("invalid header value")));
/// # drop(endpoint);
/// ```
#[inline]
pub fn matches<K, V>(name: K, value: V) -> Matches<V>
where
    HeaderName: HttpTryFrom<K>,
    <HeaderName as HttpTryFrom<K>>::Error: fmt::Debug,
    V: PartialEq<HeaderValue>,
{
    Matches {
        name: HeaderName::try_from(name).expect("invalid header name"),
        value,
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct Matches<T> {
    name: HeaderName,
    value: T,
}

mod matches {
    use super::*;

    impl<T: PartialEq<HeaderValue>> IsEndpoint for Matches<T> {}

    impl<T, Bd> Endpoint<Bd> for Matches<T>
    where
        T: PartialEq<HeaderValue> + Clone,
    {
        type Output = ();
        type Action = Oneshot<MatchesAction<T>>;

        fn action(&self) -> Self::Action {
            MatchesAction {
                name: self.name.clone(),
                value: self.value.clone(),
            }
            .into_action()
        }
    }

    #[allow(missing_debug_implementations)]
    pub struct MatchesAction<T> {
        name: HeaderName,
        value: T,
    }

    impl<T> OneshotAction for MatchesAction<T>
    where
        T: PartialEq<HeaderValue> + Clone,
    {
        type Output = ();

        fn preflight(self, cx: &mut PreflightContext<'_>) -> Result<Self::Output, Error> {
            match cx.headers().get(&self.name) {
                Some(v) if self.value == *v => Ok(()),
                _ => Err(http::StatusCode::NOT_FOUND.into()),
            }
        }
    }
}

// ==== Raw ====

/// Create an endpoint which retrieves the value of a header with the specified name.
#[inline]
pub fn raw<H>(name: H) -> Raw
where
    HeaderName: HttpTryFrom<H>,
    <HeaderName as HttpTryFrom<H>>::Error: fmt::Debug,
{
    Raw {
        name: HeaderName::try_from(name).expect("invalid header name"),
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct Raw {
    name: HeaderName,
}

mod raw {
    use super::*;

    impl IsEndpoint for Raw {}

    impl<Bd> Endpoint<Bd> for Raw {
        type Output = (Option<HeaderValue>,);
        type Action = Oneshot<RawAction>;

        fn action(&self) -> Self::Action {
            RawAction {
                name: self.name.clone(),
            }
            .into_action()
        }
    }

    #[allow(missing_debug_implementations)]
    pub struct RawAction {
        name: HeaderName,
    }

    impl OneshotAction for RawAction {
        type Output = (Option<HeaderValue>,);

        fn preflight(self, cx: &mut PreflightContext<'_>) -> Result<Self::Output, Error> {
            Ok((cx.headers().get(&self.name).cloned(),))
        }
    }
}
