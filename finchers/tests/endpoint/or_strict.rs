use finchers::endpoint::syntax;
use finchers::endpoints::{body, query};
use finchers::test;

use http::Request;
use matches::assert_matches;
use serde::de;
use serde::de::IntoDeserializer;
use std::fmt;
use std::iter::FromIterator;
use std::marker::PhantomData;

#[allow(missing_debug_implementations)]
struct CSVSeqVisitor<I, T> {
    _marker: PhantomData<fn() -> (I, T)>,
}

impl<'de, I, T> de::Visitor<'de> for CSVSeqVisitor<I, T>
where
    I: FromIterator<T>,
    T: de::Deserialize<'de>,
{
    type Value = I;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a string")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        s.split(',')
            .map(|s| de::Deserialize::deserialize(s.into_deserializer()))
            .collect()
    }
}

fn from_csv<'de, D, I, T>(de: D) -> Result<I, D::Error>
where
    D: de::Deserializer<'de>,
    I: FromIterator<T>,
    T: de::Deserialize<'de>,
{
    de.deserialize_str(CSVSeqVisitor {
        _marker: PhantomData,
    })
}

#[derive(Debug, serde::Deserialize, PartialEq)]
struct Param {
    query: String,
    count: Option<u32>,
    #[serde(deserialize_with = "from_csv", default)]
    tags: Vec<String>,
}

#[test]
fn test_or_strict() {
    let query_str = "query=rustlang&count=42&tags=tokio,hyper";

    let mut runner = test::runner({
        use finchers::endpoint::EndpointExt;

        let query = syntax::verb::get().and(query::required::<Param>());
        let form = syntax::verb::post().and(body::urlencoded::<Param>());
        query.or_strict(form)
    });

    assert_matches!(
        runner.apply(format!("/?{}", query_str)),
        Ok(ref param) if *param == Param {
            query: "rustlang".into(),
            count: Some(42),
            tags: vec!["tokio".into(), "hyper".into()]
        }
    );

    assert_matches!(
        runner.apply(Request::post("/")
            .header("content-type", "application/x-www-form-urlencoded")
            .body(query_str)),
        Ok(ref param) if *param == Param {
            query: "rustlang".into(),
            count: Some(42),
            tags: vec!["tokio".into(), "hyper".into()]
        }
    );

    assert_matches!(
        runner.apply("/"),
        Err(ref e) if e.status_code().as_u16() == 400
    );

    assert_matches!(
        runner.apply(Request::delete(format!("/?{}", query_str))),
        Err(ref e) if e.status_code().as_u16() == 405
    );

    assert_matches!(
        runner.apply(Request::put("/")
            .header("content-type", "application/x-www-form-urlencoded")
            .body(query_str)),
        Err(ref e) if e.status_code().as_u16() == 405
    );
}
