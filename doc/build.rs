extern crate skeptic;

use std::path::{Path, PathBuf};
use skeptic::*;

fn from_workspace_dir<S: AsRef<str>>(s: S) -> Vec<PathBuf> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(s.as_ref())
        .canonicalize();
    let path = match path {
        Ok(path) => path,
        Err(..) => return vec![],
    };
    markdown_files_of_directory(path.to_str().unwrap())
}

fn guide() -> Vec<PathBuf> {
    from_workspace_dir("guide/src/")
}

fn site() -> Vec<PathBuf> {
    from_workspace_dir("site")
}

fn main() {
    let mut md_files = vec![];
    md_files.extend(guide());
    md_files.extend(site());

    generate_doc_tests(&md_files);
}
