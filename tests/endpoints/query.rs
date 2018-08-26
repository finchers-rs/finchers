use finchers::endpoint::EndpointExt;
use finchers::endpoints::query;
use finchers::input::query::Serde;
use finchers::local;

use serde::Deserialize;

#[test]
fn test_query_raw() {
    let endpoint = query::raw().output::<(Option<String>,)>();

    assert_matches!(
        local::get("/?foo=bar")
            .apply(&endpoint),
        Ok((Some(ref s),)) if s == "foo=bar"
    );

    assert_matches!(local::get("/").apply(&endpoint), Ok((None,)));
}

#[test]
fn test_query_required() {
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
