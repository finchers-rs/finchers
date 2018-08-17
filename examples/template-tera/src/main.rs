#![feature(rust_2018_preview)]
#![feature(async_await, futures_api)]

use failure::{Fallible, SyncFailure};
use finchers::endpoint::{value, EndpointExt};
use finchers::error::{fail, Error};
use finchers::output::payload::Once;
use finchers::{route, routes};
use http::Response;
use std::sync::Arc;
use tera::{compile_templates, Context, Tera};

fn main() -> Fallible<()> {
    let tera = value(Arc::new(compile_templates!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/templates/**/*"
    ))));

    let index = route!(@get /)
        .and(tera.clone())
        .map(|tera| render_template(tera, "index.html"));

    let detail = route!(@get /"detail"/)
        .and(tera.clone())
        .map(|tera| render_template(tera, "detail.html"));

    let p404 = route!(@get)
        .and(tera.clone())
        .map(|tera| render_template(tera, "404.html"));

    let endpoint = routes![index, detail, p404];

    finchers::rt::launch(endpoint)?;
    Ok(())
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
