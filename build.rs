extern crate skeptic;

use skeptic::*;
use std::path::{Path, PathBuf};

fn path_string(s: &str) -> Option<String> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(s)
        .canonicalize()
        .ok()
        .and_then(|path| path.to_str().map(ToOwned::to_owned))
}

fn from_workspace_dir(s: &str) -> Vec<PathBuf> {
    path_string(s)
        .map(|path| markdown_files_of_directory(&path))
        .unwrap_or_default()
}

fn main() {
    let mut md_files = vec![];
    md_files.extend(from_workspace_dir("doc/guide/src/"));
    // md_files.extend(from_workspace_dir("doc/site"));
    // md_files.push(path_string("README.md").unwrap().into());
    generate_doc_tests(&md_files);
}
