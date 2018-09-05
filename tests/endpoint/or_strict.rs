use finchers::endpoint::syntax;
use finchers::endpoint::EndpointExt;
use finchers::endpoints::{body, query};
use finchers::input::query::{from_csv, Serde};
use finchers::local;

use matches::assert_matches;
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
struct Param {
    query: String,
    count: Option<u32>,
    #[serde(deserialize_with = "from_csv", default)]
    tags: Vec<String>,
}

#[test]
fn test_or_strict() {
    let query = syntax::verb::get().and(query::required::<Serde<Param>>());
    let form = syntax::verb::post().and(body::urlencoded::<Serde<Param>>());
    let endpoint = query.or_strict(form);

    let query_str = "query=rustlang&count=42&tags=tokio,hyper";

    assert_matches!(
        local::get(&format!("/?{}", query_str))
            .apply(&endpoint),
        Ok((Serde(ref param), )) if *param == Param {
            query: "rustlang".into(),
            count: Some(42),
            tags: vec!["tokio".into(), "hyper".into()]
        }
    );

    assert_matches!(
        local::post("/")
            .header("content-type", "application/x-www-form-urlencoded")
            .body(query_str)
            .apply(&endpoint),
        Ok((Serde(ref param), )) if *param == Param {
            query: "rustlang".into(),
            count: Some(42),
            tags: vec!["tokio".into(), "hyper".into()]
        }
    );

    assert_matches!(
        local::get("/")
            .apply(&endpoint),
        Err(ref e) if e.status_code().as_u16() == 400
    );

    assert_matches!(
        local::delete(&format!("/?{}", query_str))
            .apply(&endpoint),
        Err(ref e) if e.status_code().as_u16() == 405
    );

    assert_matches!(
        local::put("/")
            .header("content-type", "application/x-www-form-urlencoded")
            .body(query_str)
            .apply(&endpoint),
        Err(ref e) if e.status_code().as_u16() == 405
    );
}
