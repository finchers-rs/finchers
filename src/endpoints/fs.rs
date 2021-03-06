//! Endpoints for serving static contents on the file system.

use {
    crate::{
        action::{
            ActionContext, //
            EndpointAction,
            Preflight,
            PreflightContext,
        },
        endpoint::{Endpoint, IsEndpoint},
        error::{self, Error},
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
    use std::marker::PhantomData;

    impl IsEndpoint for File {}

    impl<Bd> Endpoint<Bd> for File {
        type Output = (NamedFile,);
        type Action = FileAction<Bd>;

        fn action(&self) -> Self::Action {
            FileAction {
                path: self.path.clone(),
                opening: None,
                _marker: PhantomData,
            }
        }
    }

    #[allow(missing_debug_implementations)]
    pub struct FileAction<Bd> {
        path: PathBuf,
        opening: Option<OpenNamedFile>,
        _marker: PhantomData<fn(Bd)>,
    }

    impl<Bd> EndpointAction<Bd> for FileAction<Bd> {
        type Output = (NamedFile,);

        fn poll_action(&mut self, _: &mut ActionContext<'_, Bd>) -> Poll<Self::Output, Error> {
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

        fn preflight(
            &mut self,
            cx: &mut PreflightContext<'_>,
        ) -> Result<Preflight<Self::Output>, Error> {
            let path = cx
                .cursor()
                .remaining_path()
                .percent_decode()
                .map(|path| PathBuf::from(path.into_owned()));
            let _ = cx.cursor().count();
            let path = path.map_err(error::bad_request)?;

            let mut path = self.root.join(path);
            if path.is_dir() {
                path = path.join("index.html");
            }

            self.state = State::Opening(NamedFile::open(path));
            Ok(Preflight::Incomplete)
        }

        fn poll_action(&mut self, _: &mut ActionContext<'_, Bd>) -> Poll<Self::Output, Error> {
            match self.state {
                State::Init => unreachable!(),
                State::Opening(ref mut f) => f.poll().map(|x| x.map(|x| (x,))).map_err(Into::into),
            }
        }
    }
}
