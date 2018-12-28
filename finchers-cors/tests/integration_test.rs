#[macro_use]
extern crate finchers;
extern crate finchers_cors;
extern crate http;
#[macro_use]
extern crate matches;

use finchers::local;
use finchers::prelude::*;

use http::header;

fn hello_world() -> impl_endpoint!(Output = (&'static str,)) {
    endpoint::cloned("Hello, world!").into()
}

#[test]
fn without_cors() {
    let endpoint = hello_world();

    let response = local::get("/hello")
        .header(header::HOST, "localhost")
        .respond(&endpoint);

    assert_eq!(response.status().as_u16(), 200);
    assert_eq!(response.body().to_utf8(), "Hello, world!");
}

mod simple {
    use finchers::local;
    use finchers::prelude::*;
    use finchers_cors::CorsFilter;

    use http::header;
    use http::{Method, Uri};

    use super::hello_world;

    macro_rules! cors_request {
        ({
            $(method: $method:expr,)*
            origin: $origin:expr,
        }) => {
            local::get("/hello")
                $(.method($method))*
                .header(header::HOST, "localhost:3000")
                .header(header::ORIGIN, $origin)
        };
    }

    macro_rules! assert_cors {
        ($response:expr, {
            $(origin: $origin:expr,)*
            $(allow_credentials: $true:expr,)*
        }) => {
            $( assert_cors!(@origin $response, $origin); )*
            $( assert_cors!(@allow_credentials $response, $true); )*
        };

        (@origin $response:expr, $origin:expr) => {
            assert_matches!(
                $response.headers().get(header::ACCESS_CONTROL_ALLOW_ORIGIN),
                Some(h) if h == $origin
            );
        };

        (@allow_credentials $response:expr, $true:expr) => {
            assert_matches!(
                $response.headers().get(header::ACCESS_CONTROL_ALLOW_CREDENTIALS),
                Some(h) if h == "true"
            );
        };
    }

    #[test]
    fn default() {
        let endpoint = hello_world().wrap(CorsFilter::new());

        let response = cors_request!({
            origin: "http://example.com",
        }).respond(&endpoint);

        assert_eq!(response.status().as_u16(), 200);
        assert_eq!(response.body().to_utf8(), "Hello, world!");
        assert_cors!(response, {
            origin: "*",
        });

        // without Origin
        let response = local::get("/hello")
            .header(header::HOST, "localhost:3000")
            .respond(&endpoint);
        assert_eq!(response.status().as_u16(), 400);
    }

    #[test]
    fn with_allow_origin() {
        let cors = CorsFilter::new().allow_origin(Uri::from_static("http://example.com"));
        let endpoint = hello_world().wrap(cors);

        let response = cors_request!({
            origin: "http://example.com",
        }).respond(&endpoint);
        assert_eq!(response.status().as_u16(), 200);
        assert_eq!(response.body().to_utf8(), "Hello, world!");
        assert_cors!(response, {
            origin: "http://example.com",
        });

        // disallowed Origin
        let response = cors_request!({
            origin: "http://example.org",
        }).respond(&endpoint);
        assert_eq!(response.status().as_u16(), 400);
    }

    #[test]
    fn with_allow_method() {
        let cors = CorsFilter::new().allow_method(Method::GET);
        let endpoint = hello_world().wrap(cors);

        let response = cors_request!({
            origin: "http://example.com",
        }).respond(&endpoint);

        assert_eq!(response.status().as_u16(), 200);
        assert_eq!(response.body().to_utf8(), "Hello, world!");
        assert_cors!(response, {
            origin: "*",
        });

        // disallowed Method
        let response = cors_request!({
            method: Method::DELETE,
            origin: "http://example.com",
        }).respond(&endpoint);
        assert_eq!(response.status().as_u16(), 400);
    }

    #[test]
    fn with_allow_credentials() {
        let cors = CorsFilter::new().allow_credentials(true);
        let endpoint = hello_world().wrap(cors);

        let response = cors_request!({
            origin: "http://example.com",
        }).header(header::COOKIE, "session=xxxx")
        .respond(&endpoint);

        assert_eq!(response.status().as_u16(), 200);
        assert_eq!(response.body().to_utf8(), "Hello, world!");
        assert_cors!(response, {
            origin: "http://example.com",
            allow_credentials: true,
        });
    }
}

// ==== preflight ====
mod preflight {
    use finchers::local;
    use finchers::prelude::*;
    use finchers_cors::CorsFilter;

    use http::header;
    use http::header::{HeaderName, HeaderValue};
    use http::{Method, Uri};
    use std::collections::HashSet;
    use std::time::Duration;

    use super::hello_world;

    macro_rules! preflight_request {
        ({
            origin: $origin:expr,
            method: $method:expr,
            $(headers: [$($header:expr,)*],)*
        }) => {
            local::options("/hello")
                .header(header::HOST, "localhost:3000")
                .header(header::ORIGIN, $origin)
                .header(header::ACCESS_CONTROL_REQUEST_METHOD, ($method as Method).as_str())
                $(
                    .header(header::ACCESS_CONTROL_REQUEST_HEADERS, HeaderValue::from_shared({
                        vec![$($header),*]
                            .into_iter()
                            .enumerate()
                            .fold(String::new(), |mut acc, (i, hdr): (usize, HeaderName)| {
                                if i > 0 {
                                    acc += ",";
                                }
                                acc += hdr.as_str();
                                acc
                            })
                            .into()
                    }).expect("should be a valid header value"))
                )*
        };
    }

    macro_rules! assert_preflight {
        ($response:expr, {
            $(origin: $origin:expr,)*
            $(methods: [$($method:expr,)*],)*
            $(headers: [$($header:expr,)*],)*
            $(max_age: $max_age:expr,)*
        }) => {
            assert_eq!($response.status().as_u16(), 200);
            assert_eq!($response.body().content_length(), Some(0));
            $( assert_preflight!(@origin $response, $origin); )*
            $( assert_preflight!(@method $response, [$($method),*]); )*
            $( assert_preflight!(@header $response, [$($header),*]); )*
            $( assert_preflight!(@max_age $response, $max_age); )*
        };

        (@origin $response:expr, $origin:expr) => {{
            assert_matches!(
                $response.headers().get(header::ACCESS_CONTROL_ALLOW_ORIGIN),
                Some(h) if h == $origin
            );
        }};

        (@method $response:expr, [$($method:expr),*]) => {{
            let h = $response.headers().get(header::ACCESS_CONTROL_ALLOW_METHODS).unwrap();
            let h_str = h.to_str().unwrap();
            let methods: HashSet<Method> = h_str
                .split(',')
                .map(|s| s.trim().parse())
                .collect::<Result<_, _>>()
                .unwrap();
            let expected: HashSet<Method> = vec![$($method),*].into_iter().collect();
            assert_eq!(methods, expected);
        }};

        (@header $response:expr, [$($header:expr),*]) => {{
            let h = $response.headers().get(header::ACCESS_CONTROL_ALLOW_HEADERS).unwrap();
            let h_str = h.to_str().unwrap();
            let headers: HashSet<header::HeaderName> = h_str
                .split(',')
                .map(|s| s.trim().parse())
                .collect::<Result<_, _>>()
                .unwrap();
            let expected: HashSet<header::HeaderName> = vec![$($header),*].into_iter().collect();
            assert_eq!(headers, expected);
        }};

        (@max_age $response:expr, $expected:expr) => {{
            let h = $response.headers().get(header::ACCESS_CONTROL_MAX_AGE).unwrap();
            let max_age: i64 = h.to_str().unwrap().parse().unwrap();
            assert_eq!(max_age, $expected);
        }};
    }

    #[test]
    fn default() {
        let endpoint = hello_world().wrap(CorsFilter::new());

        let response = preflight_request!({
            origin: "http://example.com",
            method: Method::GET,
        }).respond(&endpoint);

        assert_preflight!(response, {
            origin: "*",
            methods: [
                Method::GET, Method::POST, Method::PUT, Method::HEAD,
                Method::DELETE, Method::PATCH, Method::OPTIONS,
            ],
        });
    }

    #[test]
    fn with_allow_origins() {
        let cors = CorsFilter::new().allow_origin(Uri::from_static("http://example.com"));
        let endpoint = hello_world().wrap(cors);

        let response = preflight_request!({
            origin: "http://example.com",
            method: Method::GET,
        }).respond(&endpoint);
        assert_preflight!(response, {
            origin: "http://example.com",
        });

        let response = preflight_request!({
            origin: "http://example.org",
            method: Method::GET,
        }).respond(&endpoint);
        assert_eq!(response.status().as_u16(), 400);
    }

    #[test]
    fn with_allow_method() {
        let cors = CorsFilter::new().allow_method(Method::GET);
        let endpoint = hello_world().wrap(cors);

        let response = preflight_request!({
            origin: "http://example.com",
            method: Method::GET,
        }).respond(&endpoint);
        assert_preflight!(response, {
            origin: "*",
            methods: [
                Method::GET,
            ],
        });

        // disallowed method
        let response = preflight_request!({
            origin: "http://example.com",
            method: Method::POST,
        }).respond(&endpoint);
        assert_eq!(response.status().as_u16(), 400);
    }

    #[test]
    fn with_allow_headers() {
        let x_api_key = header::HeaderName::from_static("x-api-key");

        let cors = CorsFilter::new().allow_header(x_api_key.clone());
        let endpoint = hello_world().wrap(cors);

        let response = preflight_request!({
            origin: "http://example.com",
            method: Method::GET,
            headers: [
                x_api_key.clone(),
            ],
        }).respond(&endpoint);
        assert_preflight!(response, {
            origin: "*",
            headers: [
                x_api_key.clone(),
            ],
        });

        // disallowed header
        let response = preflight_request!({
            origin: "http://example.com",
            method: Method::GET,
            headers: [
                header::AUTHORIZATION,
            ],
        }).respond(&endpoint);
        assert_eq!(response.status().as_u16(), 400);
    }

    #[test]
    fn with_max_age() {
        const SECS_PER_DAY: i64 = 60 * 60 * 24;

        let cors = CorsFilter::new().max_age(Duration::from_secs(SECS_PER_DAY as u64));
        let endpoint = hello_world().wrap(cors);

        let response = preflight_request!({
            origin: "http://example.com",
            method: Method::GET,
        }).respond(&endpoint);
        assert_preflight!(response, {
            origin: "*",
            max_age: SECS_PER_DAY,
        });
    }
}
