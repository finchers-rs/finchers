use futures::future::{ok, FutureResult};
use hyper::StatusCode;

use context::Context;
use endpoint::Endpoint;
use errors::EndpointResult;
use request::Body;


pub struct TakeBody;

impl Endpoint for TakeBody {
    type Item = Body;
    type Future = FutureResult<Body, StatusCode>;

    fn apply<'r>(self, ctx: Context<'r>, mut body: Option<Body>) -> EndpointResult<'r, Self::Future> {
        if let Some(body) = body.take() {
            Ok((ctx, None, ok(body)))
        } else {
            Err(("cannot take body twice".into(), None))
        }
    }
}

pub fn take_body() -> TakeBody {
    TakeBody
}
