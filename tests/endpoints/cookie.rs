use cookie::Cookie;
use finchers::input::Cookies;
use finchers::prelude::*;
use finchers::test;
use http::Request;

#[test]
fn test_cookies_get() {
    let mut runner = test::runner({
        endpoints::cookie::cookies().map(|cookies: Cookies| cookies.get("session-id"))
    });

    assert_matches!(
        runner.apply(Request::get("/")
            .header("cookie", "session-id=xxxx")),
        Ok(Some(ref cookie))
            if cookie.name() == "session-id" &&
               cookie.value() == "xxxx"
    );
}

#[test]
fn test_cookies_add() {
    let mut runner = test::runner({
        endpoints::cookie::cookies().map(|mut cookies: Cookies| {
            cookies.add(Cookie::new("session-id", "xxxx"));
        })
    });

    let response = runner.perform("/").unwrap();

    let h_str = response
        .headers()
        .get("set-cookie")
        .expect("the header set-cookie is missing")
        .to_str()
        .unwrap();
    let cookie = Cookie::parse_encoded(h_str).expect("failed to parse Set-Cookie");

    assert_eq!(cookie.name(), "session-id");
    assert_eq!(cookie.value(), "xxxx");
}

#[test]
fn test_cookies_remove() {
    let mut runner = test::runner({
        endpoints::cookie::cookies().map(|mut cookies: Cookies| {
            cookies.remove(Cookie::named("session-id"));
        })
    });

    let response = runner
        .perform(Request::get("/").header("cookie", "session-id=xxxx"))
        .unwrap();

    let h_str = response
        .headers()
        .get("set-cookie")
        .expect("the header set-cookie is missing")
        .to_str()
        .unwrap();
    let cookie = Cookie::parse_encoded(h_str).expect("failed to parse Set-Cookie");

    assert_eq!(cookie.name(), "session-id");
    assert_eq!(cookie.value(), "");
}
