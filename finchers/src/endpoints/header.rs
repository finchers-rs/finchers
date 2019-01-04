//! Components for parsing the HTTP headers.

use {
    crate::{
        endpoint::{
            ActionContext, //
            ApplyContext,
            ApplyError,
            ApplyResult,
            Endpoint,
            EndpointAction,
            IsEndpoint,
        },
        error::{BadRequest, Error},
        input::FromHeaderValue,
    },
    futures::Poll,
    http::{
        header::{HeaderName, HeaderValue},
        HttpTryFrom,
    },
    std::{fmt, marker::PhantomData},
};

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
        type Action = ParseAction<T>;

        fn apply(&self, cx: &mut ApplyContext<'_, Bd>) -> ApplyResult<Self::Action> {
            if cx.input().headers().contains_key(&self.name) {
                Ok(ParseAction {
                    name: self.name.clone(),
                    _marker: PhantomData,
                })
            } else {
                Err(ApplyError::custom(BadRequest::from(format!(
                    "missing header: `{}'",
                    self.name.as_str()
                ))))
            }
        }
    }

    #[allow(missing_debug_implementations)]
    pub struct ParseAction<T> {
        name: HeaderName,
        _marker: PhantomData<fn() -> T>,
    }

    impl<T, Bd> EndpointAction<Bd> for ParseAction<T>
    where
        T: FromHeaderValue,
    {
        type Output = (T,);

        fn poll_action(&mut self, cx: &mut ActionContext<'_, Bd>) -> Poll<Self::Output, Error> {
            let h = cx
                .headers()
                .get(&self.name)
                .expect("The header value should be always available inside of this Action.");
            T::from_header_value(h)
                .map(|parsed| (parsed,).into())
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
        type Action = OptionalAction<T>;

        fn apply(&self, _: &mut ApplyContext<'_, Bd>) -> ApplyResult<Self::Action> {
            Ok(OptionalAction {
                name: self.name.clone(),
                _marker: PhantomData,
            })
        }
    }

    #[allow(missing_debug_implementations)]
    pub struct OptionalAction<T> {
        name: HeaderName,
        _marker: PhantomData<fn() -> T>,
    }

    impl<T, Bd> EndpointAction<Bd> for OptionalAction<T>
    where
        T: FromHeaderValue,
    {
        type Output = (Option<T>,);

        fn poll_action(&mut self, cx: &mut ActionContext<'_, Bd>) -> Poll<Self::Output, Error> {
            match cx.headers().get(&self.name) {
                Some(h) => T::from_header_value(h)
                    .map(|parsed| (Some(parsed),).into())
                    .map_err(BadRequest::from)
                    .map_err(Into::into),
                None => Ok((None,).into()),
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
        T: PartialEq<HeaderValue>,
    {
        type Output = ();
        type Action = futures::future::FutureResult<(), Error>;

        fn apply(&self, cx: &mut ApplyContext<'_, Bd>) -> ApplyResult<Self::Action> {
            match cx.headers().get(&self.name) {
                Some(v) if self.value == *v => Ok(futures::future::ok(())),
                _ => Err(ApplyError::not_matched()),
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
        type Action = RawAction;

        fn apply(&self, cx: &mut ApplyContext<'_, Bd>) -> ApplyResult<Self::Action> {
            Ok(RawAction {
                value: cx.headers().get(&self.name).cloned(),
            })
        }
    }

    #[allow(missing_debug_implementations)]
    pub struct RawAction {
        value: Option<HeaderValue>,
    }

    impl<Bd> EndpointAction<Bd> for RawAction {
        type Output = (Option<HeaderValue>,);

        fn poll_action(&mut self, _: &mut ActionContext<'_, Bd>) -> Poll<Self::Output, Error> {
            Ok((self.value.take(),).into())
        }
    }
}
