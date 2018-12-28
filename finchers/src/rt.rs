//! Components for working with Finchers runtime.

use std::cell::Cell;

use futures::sync::oneshot;
use futures::{Async, Future, Poll};
use tokio_threadpool;

use error::fail;
use error::Error;

// re-exports
#[doc(no_inline)]
pub use futures::sync::oneshot::SpawnHandle;
#[doc(no_inline)]
pub use tokio::executor::DefaultExecutor;
#[doc(no_inline)]
pub use tokio::spawn;
#[doc(no_inline)]
pub use tokio_threadpool::BlockingError;

// ====

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

/// Enter a blocking section of code.
///
/// See also the documentation of tokio-threadpool's [`blocking`] for details.
///
/// [`blocking`]: https://docs.rs/tokio-threadpool/0.1/tokio_threadpool/fn.blocking.html
pub fn blocking<R>(f: impl FnOnce() -> R) -> Poll<R, BlockingError> {
    match MODE.with(|mode| mode.get()) {
        Some(RuntimeMode::ThreadPool) | None => tokio_threadpool::blocking(f),
        Some(RuntimeMode::CurrentThread) => Ok(Async::Ready(f())),
    }
}

/// A helper function to create a future from a function which represents a blocking section.
///
/// # Example
///
/// ```
/// # #[macro_use]
/// # extern crate finchers;
/// # extern crate failure;
/// # use finchers::prelude::*;
/// # use finchers::rt::blocking_section;
/// fn get_post_sync(id: u32) -> failure::Fallible<String> {
///     // ...
/// #    drop(id);
/// #    Ok("".into())
/// }
///
/// # fn main() {
/// let endpoint = path!(@get / u32 /)
///     .and_then(|id: u32| {
///         blocking_section(move || get_post_sync(id))
///     });
/// # drop(endpoint);
/// # }
/// ```
pub fn blocking_section<F, T, E>(f: F) -> BlockingSection<F>
where
    F: FnOnce() -> Result<T, E>,
    E: Into<Error>,
{
    BlockingSection { op: Some(f) }
}

/// A `Future` which executes a blocking section with annotation.
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
        let result = try_ready!(blocking(|| {
            let op = self.op.take().unwrap();
            op()
        })
        .map_err(fail));
        result.map(Async::Ready).map_err(Into::into)
    }
}

/// Spawns a future onto the default executor and returns its handle.
#[inline]
pub fn spawn_with_handle<F>(future: F) -> SpawnHandle<F::Item, F::Error>
where
    F: Future + Send + 'static,
    F::Item: Send + 'static,
    F::Error: Send + 'static,
{
    oneshot::spawn(future, &DefaultExecutor::current())
}
