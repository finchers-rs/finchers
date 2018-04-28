#[macro_use]
extern crate finchers;
extern crate finchers_http;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate serde_json;

mod api {
    use finchers::endpoint::just;
    use finchers::endpoint::prelude::*;
    use finchers::http::StatusCode;
    use finchers::{Endpoint, Json};
    use finchers_http::json::JsonValue;

    #[derive(Debug, Default, Serialize)]
    pub struct Post {
        title: String,
        created_at: String,
        last_modified: Option<String>,
        body: String,
    }

    #[derive(Debug, Deserialize)]
    pub struct NewPost {}

    #[derive(Debug, Serialize, HttpStatus)]
    #[serde(untagged)]
    pub enum ApiResponse {
        #[status_code = "CREATED"]
        NewPost(Post),
        ThePost(Post),
    }

    pub fn endpoint() -> impl Endpoint<Output = Json<ApiResponse>> + Send + Sync + 'static {
        let post_id = param()
            .map_err(|_| bad_request("invalid post id"))
            .try_abort()
            .as_t::<u64>();

        let new_post = data()
            .map_ok(Json::into_inner)
            .map_err(|_| bad_request("invalid message body"))
            .try_abort()
            .as_t::<NewPost>();

        path("posts")
            .right(choice![
                get(post_id).map(|_id| unimplemented!()),
                get(just(())).map(|_| unimplemented!()),
                post(new_post).map(|_new_post| unimplemented!()),
            ])
            .map(|v: ApiResponse| Json::from(v))
    }

    fn bad_request(message: &str) -> JsonValue {
        JsonValue::new(
            json!({
                "error_code": "bad_request",
                "message": message,
            }),
            StatusCode::BAD_REQUEST,
        )
    }
}

fn main() {
    finchers::run(api::endpoint());
}
