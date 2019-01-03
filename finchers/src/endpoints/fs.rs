//! Endpoints for serving static contents on the file system.

use std::path::PathBuf;

use crate::endpoint::{ApplyContext, ApplyResult, Endpoint, IsEndpoint};
use crate::error::{bad_request, Error};
use crate::future::{Context, EndpointFuture, Poll};
use crate::output::fs::OpenNamedFile;
use crate::output::NamedFile;

/// Create an endpoint which serves a specified file on the file system.
#[inline]
pub fn file(path: impl Into<PathBuf>) -> File {
    File { path: path.into() }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct File {
    path: PathBuf,
}

mod file {
    use super::*;
    use futures::Future as _Future;

    impl IsEndpoint for File {}

    impl<Bd> Endpoint<Bd> for File {
        type Output = (NamedFile,);
        type Future = FileFuture;

        fn apply(&self, _: &mut ApplyContext<'_, Bd>) -> ApplyResult<Self::Future> {
            Ok(FileFuture {
                opening: NamedFile::open(self.path.clone()),
            })
        }
    }

    #[allow(missing_debug_implementations)]
    pub struct FileFuture {
        opening: OpenNamedFile,
    }

    impl<Bd> EndpointFuture<Bd> for FileFuture {
        type Output = (NamedFile,);

        fn poll_endpoint(&mut self, _: &mut Context<'_, Bd>) -> Poll<Self::Output, Error> {
            self.opening
                .poll()
                .map(|x| x.map(|x| (x,)))
                .map_err(Into::into)
        }
    }
}

/// Create an endpoint which serves files in the specified directory.
#[inline]
pub fn dir(root: impl Into<PathBuf>) -> Dir {
    Dir { root: root.into() }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct Dir {
    root: PathBuf,
}

mod dir {
    use super::*;
    use futures::Future as _Future;

    impl IsEndpoint for Dir {}

    impl<Bd> Endpoint<Bd> for Dir {
        type Output = (NamedFile,);
        type Future = DirFuture;

        fn apply(&self, cx: &mut ApplyContext<'_, Bd>) -> ApplyResult<Self::Future> {
            let path = {
                match cx.remaining_path().percent_decode() {
                    Ok(path) => Ok(PathBuf::from(path.into_owned())),
                    Err(e) => Err(e),
                }
            };
            while let Some(..) = cx.next_segment() {}

            let path = match path {
                Ok(path) => path,
                Err(e) => {
                    return Ok(DirFuture {
                        state: State::Err(Some(bad_request(e))),
                    })
                }
            };

            let mut path = self.root.join(path);
            if path.is_dir() {
                path = path.join("index.html");
            }

            Ok(DirFuture {
                state: State::Opening(NamedFile::open(path)),
            })
        }
    }

    #[allow(missing_debug_implementations)]
    pub struct DirFuture {
        state: State,
    }

    enum State {
        Err(Option<Error>),
        Opening(OpenNamedFile),
    }

    impl<Bd> EndpointFuture<Bd> for DirFuture {
        type Output = (NamedFile,);

        fn poll_endpoint(&mut self, _: &mut Context<'_, Bd>) -> Poll<Self::Output, Error> {
            match self.state {
                State::Err(ref mut err) => Err(err.take().unwrap()),
                State::Opening(ref mut f) => f.poll().map(|x| x.map(|x| (x,))).map_err(Into::into),
            }
        }
    }
}
