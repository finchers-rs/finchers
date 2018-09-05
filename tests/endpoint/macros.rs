use finchers::endpoint::syntax;
use finchers::endpoint::Endpoint;
use finchers::{path, routes};

#[test]
fn compile_test_path() {
    let _ = path!(/).with_output::<()>();
    let _ = path!(/ "foo" / i32).with_output::<(i32,)>();

    let _ = path!(@get /).with_output::<()>();
    let _ = path!(@get / "foo" / String / "bar").with_output::<(String,)>();
    let _ = path!(@get / "foo" / String / i32 / "bar" /).with_output::<(String, i32)>();
    let _ = path!(@get / i32).with_output::<(i32,)>();
    let _ = path!(@get / i32 / ).with_output::<(i32,)>();

    let _ = path!(@get / "posts" / i32 / "repo" / { syntax::remains::<String>() })
        .with_output::<(i32, String)>();
}

#[test]
fn compile_test_routes() {
    use finchers::endpoint::syntax;

    let e1 = syntax::segment("foo");
    let e2 = routes!(e1, syntax::segment("bar"), syntax::segment("baz"));
    let e3 = routes!(syntax::segment("foobar"), e2);
    let _e4 = routes!(syntax::segment("foobar"), e3,);
}
