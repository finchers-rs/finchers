extern crate finchers;

use finchers::endpoint::syntax;
use finchers::path;
use finchers::prelude::*;
use finchers::routes;

#[test]
fn test_path_macro() {
    let _ = path!(@get /);
    let _ = path!(@get / "foo" / u32);
    let _ = path!(@get / "foo" / { syntax::remains::<String>() });
}

#[test]
fn test_routes_macro() {
    let _ = routes![endpoint::unit(), endpoint::cloned(42),];
}
