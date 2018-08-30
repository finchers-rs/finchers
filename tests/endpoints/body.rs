use finchers::endpoints::body;
use finchers::local;
use serde::Deserialize;

#[test]
fn test_body_text() {
    let message = "The quick brown fox jumps over the lazy dog";

    let endpoint = body::text();

    assert_matches!(
        local::post("/").body(message).apply(&endpoint),
        Ok((ref s,)) if s == message
    );
}

#[test]
fn test_body_json() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct Param {
        text: String,
    }

    let endpoint = body::json::<Param>();

    assert_matches!(
        local::post("/")
            .header("content-type", "application/json")
            .body(r#"{ "text": "TRPL2" }"#)
            .apply(&endpoint),
        Ok((ref param,)) if *param == Param { text: "TRPL2".into() }
    );

    // missing Content-type
    assert_matches!(
        local::post("/")
            .body(r#"{ "text": "TRPL2" }"#)
            .apply(&endpoint),
        Err(ref e) if e.status_code().as_u16() == 400
    );

    // invalid content-type
    assert_matches!(
        local::post("/")
            .header("content-type", "text/plain")
            .body(r#"{ "text": "TRPL2" }"#)
            .apply(&endpoint),
        Err(ref e) if e.status_code().as_u16() == 400
    );

    // invalid data
    assert_matches!(
        local::post("/")
            .header("content-type", "application/json")
            .body(r#"invalid JSON data"#)
            .apply(&endpoint),
        Err(ref e) if e.status_code().as_u16() == 400
    );
}

#[test]
fn test_body_urlencoded() {
    use finchers::input::query::Serde;

    #[derive(Debug, PartialEq, Deserialize)]
    struct AccessTokenRequest {
        grant_type: String,
        code: String,
        redirect_uri: String,
    }

    let endpoint = body::urlencoded::<Serde<AccessTokenRequest>>();

    let form_str = r#"grant_type=authorization_code&code=SplxlOBeZQQYbYS6WxSbIA&redirect_uri=https%3A%2F%2Fclient%2Eexample%2Ecom%2Fcb"#;

    assert_matches!(
        local::post("/")
            .header("content-type", "application/x-www-form-urlencoded")
            .body(form_str)
            .apply(&endpoint),
        Ok((Serde(ref req),)) if *req == AccessTokenRequest {
            grant_type: "authorization_code".into(),
            code: "SplxlOBeZQQYbYS6WxSbIA".into(),
            redirect_uri: "https://client.example.com/cb".into(),
        }
    );

    // missing Content-type
    assert_matches!(
        local::post("/")
            .body(form_str)
            .apply(&endpoint),
        Err(ref e) if e.status_code().as_u16() == 400
    );

    // invalid content-type
    assert_matches!(
        local::post("/")
            .header("content-type", "text/plain")
            .body(form_str)
            .apply(&endpoint),
        Err(ref e) if e.status_code().as_u16() == 400
    );

    // invalid data
    assert_matches!(
        local::post("/")
            .header("content-type", "application/x-www-form-urlencoded")
            .body(r#"{ "graht_code": "authorization_code" }"#)
            .apply(&endpoint),
        Err(ref e) if e.status_code().as_u16() == 400
    );
}
