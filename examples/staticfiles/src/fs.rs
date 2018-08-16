use std::io;
use std::path::PathBuf;

use finchers::endpoint::{unit, Boxed, EndpointExt};
use finchers::endpoints::path;
use finchers::error::internal_server_error;

use crate::named_file::NamedFile;

pub fn file(path: impl Into<PathBuf>) -> Boxed<(Option<NamedFile>,)> {
    let path = path.into();
    unit()
        .and_then(async move || match await!(NamedFile::open(path.clone())) {
            Ok(f) => Ok(Some(f)),
            Err(ref e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(internal_server_error(e)),
        }).boxed()
}

pub fn dir(root: impl Into<PathBuf>) -> Boxed<(Option<NamedFile>,)> {
    let root = root.into();
    path::remains()
        .and_then(async move |path: PathBuf| {
            let mut path = root.join(path);
            if path.is_dir() {
                path = path.join("index.html");
            }

            match await!(NamedFile::open(path)) {
                Ok(f) => Ok(Some(f)),
                Err(ref e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
                Err(e) => Err(internal_server_error(e)),
            }
        }).boxed()
}
