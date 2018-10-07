use finchers::endpoint::ApplyError;
use finchers::endpoint::Endpoint;
use finchers::endpoints::query;
use finchers::test;

#[test]
fn test_query_raw() {
    let mut runner = test::runner({ query::raw().with_output::<(Option<String>,)>() });

    assert_matches!(
        runner.apply("/?foo=bar"),
        Ok(Some(ref s)) if s == "foo=bar"
    );

    assert_matches!(runner.apply("/"), Ok(None));
}

#[test]
fn test_query_parse() {
    #[derive(Debug, Deserialize)]
    struct Query {
        param: String,
        count: Option<u32>,
    }

    let mut runner = test::runner(query::required::<Query>());

    assert_matches!(
        runner.apply("/?count=20&param=rustlang"),
        Ok(ref query) if query.param == "rustlang" && query.count == Some(20)
    );

    assert_matches!(
        runner.apply("/"),
        Err(ref err) if err.is::<ApplyError>() && err.status_code().as_u16() == 400
    );
}

#[test]
fn test_query_optional() {
    #[derive(Debug, Deserialize)]
    struct Query {
        param: String,
        count: Option<u32>,
    }

    let mut runner = test::runner(query::optional::<Query>());

    assert_matches!(
        runner.apply("/?count=20&param=rustlang"),
        Ok(Some(ref query)) if query.param == "rustlang" && query.count == Some(20)
    );

    assert_matches!(runner.apply("/"), Ok(None));
}
