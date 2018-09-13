#![feature(async_await, futures_api)]

use finchers::error::{fail, Error};
use finchers::output::payload::Once;
use finchers::prelude::*;
use finchers::{path, routes};

use failure::SyncFailure;
use http::Response;
use std::sync::Arc;
use tera::{compile_templates, Context, Tera};

fn main() {
    let tera = endpoint::value(Arc::new(compile_templates!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/templates/**/*"
    ))));

    let index = path!(@get /)
        .and(tera.clone())
        .map(|tera| render_template(tera, "index.html"));

    let detail = path!(@get /"detail"/)
        .and(tera.clone())
        .map(|tera| render_template(tera, "detail.html"));

    let p404 = endpoint::syntax::verb::get()
        .and(tera.clone())
        .map(|tera| render_template(tera, "404.html"));

    let endpoint = routes![index, detail, p404];

    finchers::launch(endpoint).start("127.0.0.1:4000")
}

fn render_template(tera: Arc<Tera>, template: &str) -> Result<Response<Once<String>>, Error> {
    tera.render(template, &Context::default())
        .map(|body| {
            Response::builder()
                .header("content-type", "text/html; charset=utf-8")
                .body(Once::new(body))
                .unwrap()
        }).map_err(|err| fail(SyncFailure::new(err)))
}
