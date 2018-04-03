use std::borrow::Cow;
use std::io;
use futures::{Poll, Stream};

/// An abstruction of response body.
pub trait ResponseBody {
    /// The type of items in `Stream`
    type Item: AsRef<[u8]> + 'static;

    /// The type of value returned from `into_stream`
    type Stream: Stream<Item = Self::Item, Error = io::Error> + 'static;

    /// Convert itself into an `Stream`.
    fn into_stream(self) -> Self::Stream;
}

#[allow(missing_debug_implementations)]
#[derive(Default)]
pub struct Body<T: ?Sized + ToOwned + AsRef<[u8]> + 'static> {
    inner: Option<Cow<'static, T>>,
}

impl<S, T> From<S> for Body<T>
where
    S: Into<Cow<'static, T>>,
    T: ?Sized + ToOwned + AsRef<[u8]> + 'static,
{
    fn from(body: S) -> Self {
        Body {
            inner: Some(body.into()),
        }
    }
}

impl<T> Stream for Body<T>
where
    T: ?Sized + ToOwned + AsRef<[u8]> + 'static,
{
    type Item = BodyItem<T>;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        Ok(self.inner.take().map(BodyItem).into())
    }
}

impl<T> ResponseBody for Body<T>
where
    T: ?Sized + ToOwned + AsRef<[u8]> + 'static,
{
    type Item = BodyItem<T>;
    type Stream = Self;

    #[inline]
    fn into_stream(self) -> Self::Stream {
        self
    }
}

#[allow(missing_debug_implementations)]
pub struct BodyItem<T: ?Sized + ToOwned + AsRef<[u8]> + 'static>(Cow<'static, T>);

impl<T> AsRef<[u8]> for BodyItem<T>
where
    T: ?Sized + ToOwned + AsRef<[u8]> + 'static,
{
    #[inline]
    fn as_ref(&self) -> &[u8] {
        (*self.0).as_ref()
    }
}

macro_rules! impl_response_body {
    ($(
        $a:ty => ($($t:ty),*);
    )*) => {$(
        $(
            impl ResponseBody for $t {
                type Item = BodyItem<$a>;
                type Stream = Body<$a>;

                #[inline]
                fn into_stream(self) -> Self::Stream {
                    Body::from(self)
                }
            }
        )*
    )*};
}

impl_response_body! {
    str => (String, &'static str, Cow<'static, str>);
    [u8] => (Vec<u8>, &'static [u8], Cow<'static, [u8]>);
}
