#[macro_use]
extern crate finchers;
extern crate http;

use finchers::output::HttpStatus;
use http::StatusCode;

fn assert_impl_http_response<T: HttpStatus>(t: T) -> T {
    t
}

#[test]
fn derive_http_response_for_normal_struct() {
    #[derive(HttpStatus)]
    #[status_code = "NOT_FOUND"]
    struct Param {}
    let param = assert_impl_http_response(Param {});
    assert_eq!(param.status_code(), StatusCode::NOT_FOUND);
}

#[test]
fn derive_http_response_for_generic_struct() {
    #[derive(HttpStatus)]
    struct Param<T> {
        _value: T,
    }
    let param = assert_impl_http_response(Param { _value: 0usize });
    assert_eq!(param.status_code(), StatusCode::OK);
}

#[test]
fn derive_http_response_for_enum() {
    #[derive(HttpStatus)]
    #[status_code = "CREATED"]
    enum Param {
        A(u32),
        B {
            _id: u32,
        },
        #[status_code = "FOUND"]
        C,
    }

    let param = assert_impl_http_response(Param::A(0));
    assert_eq!(param.status_code(), StatusCode::CREATED);

    let param = assert_impl_http_response(Param::B { _id: 0 });
    assert_eq!(param.status_code(), StatusCode::CREATED);

    let param = assert_impl_http_response(Param::C);
    assert_eq!(param.status_code(), StatusCode::FOUND);
}
