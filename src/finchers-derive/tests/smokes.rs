extern crate finchers;
#[macro_use]
extern crate finchers_derive;

use finchers::core::HttpStatus;
use finchers::http::StatusCode;

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
    enum Param {
        #[status_code = "FOUND"]
        A,
        B,
    }

    let param = assert_impl_http_response(Param::A);
    assert_eq!(param.status_code(), StatusCode::FOUND);

    let param = assert_impl_http_response(Param::B);
    assert_eq!(param.status_code(), StatusCode::OK);
}
