use finchers::endpoint::syntax;
use finchers::prelude::*;
use finchers::test;
use matches::assert_matches;

#[test]
fn test_match_single_segment() {
    let mut runner = test::runner(syntax::segment("foo"));

    assert_matches!(runner.apply_raw("/foo"), Ok(()));
    assert_matches!(runner.apply_raw("/bar"), Err(..));
}

#[test]
fn test_match_multi_segments() {
    let mut runner = test::runner({ syntax::segment("foo").and(syntax::segment("bar")) });

    assert_matches!(runner.apply_raw("/foo/bar"), Ok(()));
    assert_matches!(runner.apply_raw("/foo/bar/"), Ok(()));
    assert_matches!(runner.apply_raw("/foo/bar/baz"), Ok(()));
    assert_matches!(runner.apply_raw("/foo"), Err(..));
    assert_matches!(runner.apply_raw("/foo/baz"), Err(..));
}

#[test]
fn test_match_encoded_path() {
    let mut runner = test::runner(syntax::segment("foo/bar"));

    assert_matches!(runner.apply_raw("/foo%2Fbar"), Ok(()));
    assert_matches!(runner.apply_raw("/foo/bar"), Err(..));
}

#[test]
fn test_extract_integer() {
    let mut runner = test::runner(syntax::param::<i32>());

    assert_matches!(runner.apply("/42"), Ok(42i32));
    assert_matches!(runner.apply("/foo"), Err(..));
}

#[test]
fn test_extract_strings() {
    let mut runner = test::runner(syntax::segment("foo").and(syntax::remains::<String>()));

    assert_matches!(
        runner.apply("/foo/bar/baz/"),
        Ok(ref s) if s == "bar/baz/"
    );
}

// #[test]
// fn test_path_macro() {
//     let mut runner = test::runner(
//         path!(@get / "posts" / u32 / "stars" /)
//             .map(|id: u32| format!("id={}", id))
//             .with_output::<(String,)>(),
//     );
//     assert_matches!(
//         runner.apply("/posts/42/stars"),
//         Ok(ref s) if s == "id=42"
//     );
// }
