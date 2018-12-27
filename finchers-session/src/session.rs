use finchers::endpoint;
use finchers::error::Error;
use finchers::input::Input;

use futures::{Future, IntoFuture, Poll};

/// The trait representing the backend to manage session value.
#[allow(missing_docs)]
pub trait RawSession {
    type WriteFuture: Future<Item = (), Error = Error>;

    fn get(&self) -> Option<&str>;
    fn set(&mut self, value: String);
    fn remove(&mut self);
    fn write(self, input: &mut Input) -> Self::WriteFuture;
}

/// A struct which manages the session value per request.
#[derive(Debug)]
#[must_use = "The value must be convert into a Future to finish the session handling."]
pub struct Session<S: RawSession> {
    raw: S,
}

impl<S> Session<S>
where
    S: RawSession,
{
    #[allow(missing_docs)]
    pub fn new(raw: S) -> Session<S> {
        Session { raw }
    }

    /// Get the session value if available.
    pub fn get(&self) -> Option<&str> {
        self.raw.get()
    }

    /// Set the session value.
    pub fn set(&mut self, value: impl Into<String>) {
        self.raw.set(value.into());
    }

    /// Annotates to remove session value to the backend.
    pub fn remove(&mut self) {
        self.raw.remove();
    }

    #[allow(missing_docs)]
    pub fn with<R>(
        mut self,
        f: impl FnOnce(&mut Self) -> R,
    ) -> impl Future<Item = R::Item, Error = Error>
    where
        R: IntoFuture<Error = Error>,
    {
        f(&mut self)
            .into_future()
            .and_then(move |item| self.into_future().map(move |()| item))
    }
}

impl<S> IntoFuture for Session<S>
where
    S: RawSession,
{
    type Item = ();
    type Error = Error;
    type Future = WriteSessionFuture<S::WriteFuture>;

    fn into_future(self) -> Self::Future {
        WriteSessionFuture {
            future: endpoint::with_get_cx(|input| self.raw.write(input)),
        }
    }
}

#[derive(Debug)]
#[must_use = "futures do not anything unless polled."]
pub struct WriteSessionFuture<F> {
    future: F,
}

impl<F> Future for WriteSessionFuture<F>
where
    F: Future<Item = ()>,
    F::Error: Into<Error>,
{
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.future.poll().map_err(Into::into)
    }
}
