//! Components for managing blocking codes.

use std::cell::Cell;

use futures::{Async, Future, Poll};
use tokio_threadpool;
use tokio_threadpool::BlockingError;

use error::fail;
use error::Error;

#[derive(Debug, Copy, Clone)]
pub(crate) enum RuntimeMode {
    ThreadPool,
    CurrentThread,
}

thread_local!(static MODE: Cell<Option<RuntimeMode>> = Cell::new(None));

pub(crate) fn with_set_runtime_mode<R>(mode: RuntimeMode, f: impl FnOnce() -> R) -> R {
    #[allow(missing_debug_implementations)]
    struct SetOnDrop(Option<RuntimeMode>);

    impl Drop for SetOnDrop {
        fn drop(&mut self) {
            MODE.with(|mode| mode.set(self.0));
        }
    }

    let mode = MODE.with(|m| m.replace(Some(mode)));
    let _prev = SetOnDrop(mode);
    match mode {
        Some(..) => panic!("The runtime mode has already set on the current context."),
        None => f(),
    }
}

fn with_get_runtime_mode<R>(f: impl FnOnce(RuntimeMode) -> R) -> R {
    match MODE.with(|m| m.get()) {
        Some(mode) => f(mode),
        None => panic!("The runtime mode is not set on the current context."),
    }
}

/// Enter a blocking section of code.
///
/// See also the documentation of tokio-threadpool's [`blocking`] for details.
///
/// [`blocking`]: https://docs.rs/tokio-threadpool/0.1/tokio_threadpool/fn.blocking.html
pub fn blocking<R>(f: impl FnOnce() -> R) -> Poll<R, BlockingError> {
    with_get_runtime_mode(|mode| match mode {
        RuntimeMode::ThreadPool => tokio_threadpool::blocking(f),
        RuntimeMode::CurrentThread => Ok(Async::Ready(f())),
    })
}

/// A helper function to create a future from a function which represents a blocking section.
///
/// # Example
///
/// ```ignore
/// path!(@get / u32 /)
///     .and_then(|id: u32| blocking_section(|| {
///         get_post_sync(id).map_err(finchers::error::fail)
///     }))
/// ```
pub fn blocking_section<F, T, E>(f: F) -> BlockingSection<F>
where
    F: FnOnce() -> Result<T, E>,
    E: Into<Error>,
{
    BlockingSection { op: Some(f) }
}

#[derive(Debug)]
pub struct BlockingSection<F> {
    op: Option<F>,
}

impl<F, T, E> Future for BlockingSection<F>
where
    F: FnOnce() -> Result<T, E>,
    E: Into<Error>,
{
    type Item = T;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let result = try_ready!(
            blocking(|| {
                let op = self.op.take().unwrap();
                op()
            }).map_err(fail)
        );
        result.map(Async::Ready).map_err(Into::into)
    }
}

pub fn blocking_section_ok<F, R>(f: F) -> BlockingSection<impl FnOnce() -> Result<R, Error>>
where
    F: FnOnce() -> R,
{
    blocking_section(move || Ok(f()))
}
