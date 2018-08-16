//! Endpoints for serving static contents on the file system.

use std::io;
use std::path::PathBuf;

use futures_util::future::FutureExt;

use crate::endpoint::{unit, Boxed, EndpointExt};
use crate::endpoints::path;
use crate::error::internal_server_error;
use crate::output::NamedFile;

/// Create an endpoint which serves a specified file on the file system.
pub fn file(path: impl Into<PathBuf>) -> Boxed<(Option<NamedFile>,)> {
    let path = path.into();
    unit()
        .and_then(move || {
            NamedFile::open(path.clone()).map(|file| match file {
                Ok(f) => Ok(Some(f)),
                Err(ref e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
                Err(e) => Err(internal_server_error(e)),
            })
        }).boxed()
}

/// Create an endpoint which serves files in the specified directory.
pub fn dir(root: impl Into<PathBuf>) -> Boxed<(Option<NamedFile>,)> {
    let root = root.into();
    path::remains()
        .and_then(move |path: PathBuf| {
            let mut path = root.join(path);
            if path.is_dir() {
                path = path.join("index.html");
            }

            NamedFile::open(path).map(|file| match file {
                Ok(f) => Ok(Some(f)),
                Err(ref e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
                Err(e) => Err(internal_server_error(e)),
            })
        }).boxed()
}
