extern crate finchers;
extern crate finchers_juniper;
extern crate juniper;
#[macro_use]
extern crate percent_encoding;
extern crate http;

use finchers::endpoint::syntax;
use finchers::prelude::*;
use finchers::test;
use finchers::test::{TestResult, TestRunner};
use finchers_juniper::request::{GraphQLRequest, GraphQLResponse};

use juniper::http::tests as http_tests;
use juniper::tests::model::Database;
use juniper::{EmptyMutation, RootNode};

use http::{Request, Response};
use percent_encoding::{utf8_percent_encode, QUERY_ENCODE_SET};
use std::cell::RefCell;

type Schema = RootNode<'static, Database, EmptyMutation<Database>>;

struct TestFinchersIntegration<E> {
    runner: RefCell<TestRunner<E>>,
}

impl<E> http_tests::HTTPIntegration for TestFinchersIntegration<E>
where
    for<'e> E: Endpoint<'e, Output = (GraphQLResponse,)>,
{
    fn get(&self, url: &str) -> http_tests::TestResponse {
        let response = self
            .runner
            .borrow_mut()
            .perform(Request::get(custom_url_encode(url)))
            .unwrap();
        make_test_response(response)
    }

    fn post(&self, url: &str, body: &str) -> http_tests::TestResponse {
        let response = self
            .runner
            .borrow_mut()
            .perform(
                Request::post(custom_url_encode(url))
                    .header("content-type", "application/json")
                    .body(body.to_owned()),
            ).unwrap();
        make_test_response(response)
    }
}

fn custom_url_encode(url: &str) -> String {
    define_encode_set!{
        pub CUSTOM_ENCODE_SET = [QUERY_ENCODE_SET] | {'{', '}'}
    }
    utf8_percent_encode(url, CUSTOM_ENCODE_SET).to_string()
}

fn make_test_response(response: Response<TestResult>) -> http_tests::TestResponse {
    let status_code = response.status().as_u16() as i32;
    let content_type = response
        .headers()
        .get("content-type")
        .expect("No content type header from endpoint")
        .to_str()
        .expect("failed to convert the header value to string")
        .to_owned();
    let body = response.body().to_utf8().unwrap().into_owned();
    http_tests::TestResponse {
        status_code,
        content_type,
        body: Some(body),
    }
}

#[test]
fn test_finchers_integration() {
    let database = Database::new();
    let schema = Schema::new(Database::new(), EmptyMutation::<Database>::new());
    let endpoint = syntax::eos()
        .and(finchers_juniper::graphql_request())
        .and(endpoint::by_ref(database))
        .and(endpoint::by_ref(schema))
        .and_then(
            |req: GraphQLRequest, db: &Database, schema: &Schema| Ok(req.execute(schema, db)),
        );
    let integration = TestFinchersIntegration {
        runner: RefCell::new(test::runner(endpoint)),
    };
    http_tests::run_http_test_suite(&integration);
}
