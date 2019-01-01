//! Endpoints for serving static contents on the file system.

use futures::{Future, Poll};
use std::path::PathBuf;

use crate::endpoint::Endpoint;
use crate::error::{bad_request, Error};
use crate::future::EndpointFuture;
use crate::output::fs::OpenNamedFile;
use crate::output::NamedFile;

/// Create an endpoint which serves a specified file on the file system.
#[inline]
pub fn file(
    path: impl Into<PathBuf>,
) -> impl Endpoint<
    Output = (NamedFile,),
    Future = impl EndpointFuture<Output = (NamedFile,)> + Send + 'static,
> {
    let path = path.into();
    crate::endpoint::apply_fn(move |_| {
        Ok(FileFuture {
            state: State::Opening(NamedFile::open(path.clone())),
        })
    })
}

/// Create an endpoint which serves files in the specified directory.
#[inline]
pub fn dir(
    root: impl Into<PathBuf>,
) -> impl Endpoint<
    Output = (NamedFile,),
    Future = impl EndpointFuture<Output = (NamedFile,)> + Send + 'static, //
> {
    let root = root.into();
    crate::endpoint::apply_fn(move |ecx| {
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

        let mut path = root.join(path);
        if path.is_dir() {
            path = path.join("index.html");
        }

        Ok(FileFuture {
            state: State::Opening(NamedFile::open(path)),
        })
    })
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
