//! Endpoints for serving static contents on the file system.

use std::io;
use std::mem::PinMut;
use std::path::PathBuf;
use std::task;
use std::task::Poll;

use futures_core::future::Future;
use pin_utils::unsafe_unpinned;

use crate::endpoint::Endpoint;
use crate::error::{bad_request, internal_server_error, Error, NoRoute};
use crate::generic::{one, One};
use crate::input::{Cursor, Input};
use crate::output::fs::OpenNamedFile;
use crate::output::NamedFile;

/// Create an endpoint which serves a specified file on the file system.
pub fn file(path: impl Into<PathBuf>) -> File {
    File { path: path.into() }
}

#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct File {
    path: PathBuf,
}

impl Endpoint for File {
    type Output = One<NamedFile>;
    type Future = FileFuture;

    fn apply<'c>(&self, _: PinMut<'_, Input>, c: Cursor<'c>) -> Option<(Self::Future, Cursor<'c>)> {
        Some((
            FileFuture {
                state: State::Opening(NamedFile::open(self.path.clone())),
            },
            c,
        ))
    }
}

/// Create an endpoint which serves files in the specified directory.
pub fn dir(root: impl Into<PathBuf>) -> Dir {
    Dir { root: root.into() }
}

#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct Dir {
    root: PathBuf,
}

impl Endpoint for Dir {
    type Output = One<NamedFile>;
    type Future = FileFuture;

    fn apply<'c>(
        &self,
        _: PinMut<'_, Input>,
        mut cursor: Cursor<'c>,
    ) -> Option<(Self::Future, Cursor<'c>)> {
        let path = cursor.remaining_path().percent_decode();
        let _ = cursor.by_ref().count();
        let path = match path {
            Ok(path) => PathBuf::from(path.into_owned()),
            Err(e) => {
                return Some((
                    FileFuture {
                        state: State::Err(Some(bad_request(e))),
                    },
                    cursor,
                ))
            }
        };

        let mut path = self.root.join(path);
        if path.is_dir() {
            path = path.join("index.html");
        }

        Some((
            FileFuture {
                state: State::Opening(NamedFile::open(path)),
            },
            cursor,
        ))
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct FileFuture {
    state: State,
}

#[derive(Debug)]
enum State {
    Err(Option<Error>),
    Opening(OpenNamedFile),
}

impl FileFuture {
    unsafe_unpinned!(state: State);
}

impl Future for FileFuture {
    type Output = Result<One<NamedFile>, Error>;

    fn poll(mut self: PinMut<'_, Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        match self.state() {
            State::Err(ref mut err) => Poll::Ready(Err(err.take().unwrap())),
            State::Opening(ref mut f) => {
                let f = unsafe { PinMut::new_unchecked(f) };
                f.poll(cx).map_ok(one).map_err(io_error)
            }
        }
    }
}

// TODO: impl HttpError for io::Error
fn io_error(err: io::Error) -> Error {
    match err {
        ref e if e.kind() == io::ErrorKind::NotFound => Error::from(NoRoute),
        e => internal_server_error(e),
    }
}
