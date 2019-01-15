use finchers::endpoint::syntax;

#[test]
fn compile_test_path() {
    let _ = syntax::path!("/");
    let _ = syntax::path!("/foo");
    let _ = syntax::path!("/foo/");
    let _ = syntax::path!("/foo/<i32>");
    let _ = syntax::path!("/foo/<..std::path::PathBuf>");

    let _ = syntax::path!(@get "/");
    let _ = syntax::path!(@get "/foo/<String>/bar");
    let _ = syntax::path!(@get "/foo/<String>/<i32>/bar/");
    let _ = syntax::path!(@get "/<i32>");
    let _ = syntax::path!(@get "/<i32>/");
    let _ = syntax::path!(@get "/posts/<i32>/repo/<..String>");
}

// #[test]
// fn compile_test_routes() {
//     use finchers::endpoint::syntax;

//     let e1 = syntax::segment("foo");
//     let e2 = routes!(e1, syntax::segment("bar"), syntax::segment("baz"));
//     let e3 = routes!(syntax::segment("foobar"), e2);
//     let _e4 = routes!(syntax::segment("foobar"), e3,);
// }

#[test]
fn test_extract_path_statics() {
    let mut runner = finchers::test::runner({
        syntax::path!("/foo/bar") //
    });

    matches::assert_matches!(runner.apply_raw("/foo/bar"), Ok(()));
    matches::assert_matches!(runner.apply_raw("/"), Err(..));
}

#[test]
fn test_extract_path_single_param() {
    let mut runner = finchers::test::runner({ syntax::path!("/foo/<i32>") });

    matches::assert_matches!(runner.apply("/foo/42"), Ok(42_i32));
    matches::assert_matches!(runner.apply("/"), Err(..));
    matches::assert_matches!(runner.apply("/foo/bar"), Err(..));
}

#[test]
fn test_extract_path_catch_all_param() {
    let mut runner = finchers::test::runner({ syntax::path!("/foo/<..String>") });

    assert_eq!(runner.apply("/foo/").ok(), Some("".into()));
    assert_eq!(runner.apply("/foo/bar/baz").ok(), Some("bar/baz".into()));
    matches::assert_matches!(runner.apply("/"), Err(..));
}
