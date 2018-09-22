//! Endpoints for serving static contents on the file system.

use futures::{Future, Poll};
use std::path::PathBuf;

use crate::endpoint::{Context, Endpoint, EndpointResult};
use crate::error::{bad_request, Error};
use crate::output::fs::OpenNamedFile;
use crate::output::NamedFile;

/// Create an endpoint which serves a specified file on the file system.
#[inline]
pub fn file(path: impl Into<PathBuf>) -> File {
    (File { path: path.into() }).with_output::<(NamedFile,)>()
}

#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct File {
    path: PathBuf,
}

impl<'a> Endpoint<'a> for File {
    type Output = (NamedFile,);
    type Future = FileFuture;

    fn apply(&self, _: &mut Context<'_>) -> EndpointResult<Self::Future> {
        Ok(FileFuture {
            state: State::Opening(NamedFile::open(self.path.clone())),
        })
    }
}

/// Create an endpoint which serves files in the specified directory.
#[inline]
pub fn dir(root: impl Into<PathBuf>) -> Dir {
    (Dir { root: root.into() }).with_output::<(NamedFile,)>()
}

#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct Dir {
    root: PathBuf,
}

impl<'a> Endpoint<'a> for Dir {
    type Output = (NamedFile,);
    type Future = FileFuture;

    fn apply(&self, ecx: &mut Context<'_>) -> EndpointResult<Self::Future> {
        let path = {
            match ecx.remaining_path().percent_decode() {
                Ok(path) => Ok(PathBuf::from(path.into_owned())),
                Err(e) => Err(e),
            }
        };
        while let Some(..) = ecx.next_segment() {}

        let path = match path {
            Ok(path) => path,
            Err(e) => {
                return Ok(FileFuture {
                    state: State::Err(Some(bad_request(e))),
                })
            }
        };

        let mut path = self.root.join(path);
        if path.is_dir() {
            path = path.join("index.html");
        }

        Ok(FileFuture {
            state: State::Opening(NamedFile::open(path)),
        })
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

impl Future for FileFuture {
    type Item = (NamedFile,);
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match self.state {
            State::Err(ref mut err) => Err(err.take().unwrap()),
            State::Opening(ref mut f) => f.poll().map(|x| x.map(|x| (x,))).map_err(Into::into),
        }
    }
}
