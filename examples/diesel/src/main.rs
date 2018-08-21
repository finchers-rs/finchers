#![feature(
    pin,
    arbitrary_self_types,
    async_await,
    await_macro,
    futures_api
)]

#[macro_use]
extern crate diesel;

mod api;
mod database;
mod model;
mod schema;

use failure::Fallible;
use serde::Deserialize;
use std::env;

use finchers::endpoint::{lazy, EndpointExt};
use finchers::endpoints::{body, query};
use finchers::{route, routes};

use crate::database::ConnectionPool;

fn main() -> Fallible<()> {
    dotenv::dotenv()?;

    let pool = ConnectionPool::init(env::var("DATABASE_URL")?)?;
    let acquire_conn = lazy(move |_| {
        let fut = pool.acquire_conn();
        async move { await!(fut).map_err(Into::into) }
    });

    let endpoint = route!(/"api"/"v1"/"posts").and(routes!{
        route!(@get /)
            .and(query::parse())
            .and(acquire_conn.clone())
            .and_then(crate::api::get_posts),

        route!(@post /)
            .and(body::json())
            .and(acquire_conn.clone())
            .and_then(crate::api::create_post),

        route!(@get / i32 /)
            .and(acquire_conn.clone())
            .and_then(crate::api::find_post),
    });

    finchers::launch(endpoint).start("127.0.0.1:4000");
    Ok(())
}
