mod endpoint;
mod endpoints;

#[test]
fn test_perform_on_error_response() {
    use finchers::prelude::*;
    use finchers::test;

    let mut runner = test::runner({
        endpoint::lazy(|| Err::<&str, _>(finchers::error::bad_request("error"))) //
    });

    let response = runner.perform("/").unwrap();
    assert_eq!(response.status().as_u16(), 400);
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
