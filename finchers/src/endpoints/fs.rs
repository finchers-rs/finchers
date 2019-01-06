//! Endpoints for serving static contents on the file system.

use {
    crate::{
        endpoint::{
            ActionContext, //
            ApplyContext,
            Endpoint,
            EndpointAction,
            IsEndpoint,
            Preflight,
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

        fn action(&self) -> Self::Action {
            FileAction {
                path: self.path.clone(),
                opening: None,
            }
        }
    }

    #[allow(missing_debug_implementations)]
    pub struct FileAction {
        path: PathBuf,
        opening: Option<OpenNamedFile>,
    }

    impl<Bd> EndpointAction<Bd> for FileAction {
        type Output = (NamedFile,);
        type Error = Error;

        fn poll_action(
            &mut self,
            _: &mut ActionContext<'_, Bd>,
        ) -> Poll<Self::Output, Self::Error> {
            loop {
                if let Some(ref mut opening) = self.opening {
                    return opening.poll().map(|x| x.map(|x| (x,))).map_err(Into::into);
                }
                self.opening = Some(NamedFile::open(self.path.clone()));
            }
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

        fn action(&self) -> Self::Action {
            DirAction {
                root: self.root.clone(),
                state: State::Init,
            }
        }
    }

    #[allow(missing_debug_implementations)]
    pub struct DirAction {
        root: PathBuf,
        state: State,
    }

    enum State {
        Init,
        Opening(OpenNamedFile),
    }

    impl<Bd> EndpointAction<Bd> for DirAction {
        type Output = (NamedFile,);
        type Error = Error;

        fn preflight(
            &mut self,
            cx: &mut ApplyContext<'_>,
        ) -> Result<Preflight<Self::Output>, Self::Error> {
            let path = {
                match cx.remaining_path().percent_decode() {
                    Ok(path) => Ok(PathBuf::from(path.into_owned())),
                    Err(e) => Err(e),
                }
            };
            let _ = cx.by_ref().count();

            let path = match path {
                Ok(path) => path,
                Err(e) => return Err(BadRequest::from(e).into()),
            };

            let mut path = self.root.join(path);
            if path.is_dir() {
                path = path.join("index.html");
            }

            self.state = State::Opening(NamedFile::open(path));
            Ok(Preflight::Incomplete)
        }

        fn poll_action(
            &mut self,
            _: &mut ActionContext<'_, Bd>,
        ) -> Poll<Self::Output, Self::Error> {
            match self.state {
                State::Init => unreachable!(),
                State::Opening(ref mut f) => f.poll().map(|x| x.map(|x| (x,))).map_err(Into::into),
            }
        }
    }
}
