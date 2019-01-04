//! Endpoints for serving static contents on the file system.

use {
    crate::{
        endpoint::{
            ActionContext, //
            Apply,
            ApplyContext,
            Endpoint,
            EndpointAction,
            IsEndpoint,
        },
        error::{BadRequest, Error},
        output::fs::{NamedFile, OpenNamedFile},
    },
    futures::Poll,
    std::path::PathBuf,
};

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
        type Error = Error;
        type Action = FileAction;

        fn apply(&self, _: &mut ApplyContext<'_, Bd>) -> Apply<Bd, Self> {
            Ok(FileAction {
                opening: NamedFile::open(self.path.clone()),
            })
        }
    }

    #[allow(missing_debug_implementations)]
    pub struct FileAction {
        opening: OpenNamedFile,
    }

    impl<Bd> EndpointAction<Bd> for FileAction {
        type Output = (NamedFile,);
        type Error = Error;

        fn poll_action(
            &mut self,
            _: &mut ActionContext<'_, Bd>,
        ) -> Poll<Self::Output, Self::Error> {
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
        type Error = Error;
        type Action = DirAction;

        fn apply(&self, cx: &mut ApplyContext<'_, Bd>) -> Apply<Bd, Self> {
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
                    return Ok(DirAction {
                        state: State::Err(Some(BadRequest::from(e).into())),
                    })
                }
            };

            let mut path = self.root.join(path);
            if path.is_dir() {
                path = path.join("index.html");
            }

            Ok(DirAction {
                state: State::Opening(NamedFile::open(path)),
            })
        }
    }

    #[allow(missing_debug_implementations)]
    pub struct DirAction {
        state: State,
    }

    enum State {
        Err(Option<Error>),
        Opening(OpenNamedFile),
    }

    impl<Bd> EndpointAction<Bd> for DirAction {
        type Output = (NamedFile,);
        type Error = Error;

        fn poll_action(
            &mut self,
            _: &mut ActionContext<'_, Bd>,
        ) -> Poll<Self::Output, Self::Error> {
            match self.state {
                State::Err(ref mut err) => Err(err.take().unwrap()),
                State::Opening(ref mut f) => f.poll().map(|x| x.map(|x| (x,))).map_err(Into::into),
            }
        }
    }
}
