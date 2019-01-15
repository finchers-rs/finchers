mod endpoint;
mod endpoints;

#[test]
fn version_sync() {
    version_sync::assert_html_root_url_updated!("src/lib.rs");
    version_sync::assert_markdown_deps_updated!("README.md");
}

// #[test]
// fn test_path_macro() {
//     let _ = path!(@get /);
//     let _ = path!(@get / "foo" / u32);
//     let _ = path!(@get / "foo" / { syntax::remains::<String>() });
// }

// #[test]
// fn test_routes_macro() {
//     let _ = routes![endpoint::unit(), endpoint::value(42),];
// }
