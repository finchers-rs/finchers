use finchers::endpoint::Endpoint;
use finchers::endpoint::EndpointError;
use finchers::endpoints::query;
use finchers::input::query::Serde;
use finchers::local;

#[test]
fn test_query_raw() {
    let endpoint = query::raw().with_output::<(Option<String>,)>();

    assert_matches!(
        local::get("/?foo=bar")
            .apply(&endpoint),
        Ok((Some(ref s),)) if s == "foo=bar"
    );

    assert_matches!(local::get("/").apply(&endpoint), Ok((None,)));
}

#[test]
fn test_query_parse() {
    #[derive(Debug, Deserialize)]
    struct Query {
        param: String,
        count: Option<u32>,
    }

    let endpoint = query::required::<Serde<Query>>();

    assert_matches!(
        local::get("/?count=20&param=rustlang")
            .apply(&endpoint),
        Ok((ref query,)) if query.param == "rustlang" && query.count == Some(20)
    );

    assert_matches!(
        local::get("/")
            .apply(&endpoint),
        Err(ref err) if err.is::<EndpointError>() && err.status_code().as_u16() == 400
    );
}

#[test]
fn test_query_optional() {
    #[derive(Debug, Deserialize)]
    struct Query {
        param: String,
        count: Option<u32>,
    }

    let endpoint = query::optional::<Serde<Query>>();

    assert_matches!(
        local::get("/?count=20&param=rustlang")
            .apply(&endpoint),
        Ok((Some(ref query),)) if query.param == "rustlang" && query.count == Some(20)
    );

    assert_matches!(local::get("/").apply(&endpoint), Ok((None,)));
}
