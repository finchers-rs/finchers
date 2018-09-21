#![allow(proc_macro_derive_resolution_fallback)]

#[macro_use]
extern crate diesel;

mod api;
mod database;
mod model;
mod schema;

use failure::Fallible;
use futures::prelude::*;
use http::StatusCode;
use serde::Deserialize;
use std::env;

use finchers::input::query::Serde;
use finchers::prelude::*;
use finchers::{output, path, routes};

use crate::database::ConnectionPool;

fn main() -> Fallible<()> {
    dotenv::dotenv()?;

    let pool = ConnectionPool::init(env::var("DATABASE_URL")?)?;
    let acquire_conn = endpoint::unit().and_then(move || pool.acquire_conn().map_err(Into::into));

    let endpoint = path!(/"api"/"v1"/"posts").and(routes!{
        path!(@get /)
            .and(endpoints::query::optional().map(|query: Option<_>| {
                query.map(Serde::into_inner)
            }))
            .and(acquire_conn.clone())
            .and_then(|query, conn| crate::api::get_posts(query, conn).from_err())
            .map(output::Json),

        path!(@post /)
            .and(endpoints::body::json())
            .and(acquire_conn.clone())
            .and_then(|new_post, conn| crate::api::create_post(new_post, conn).from_err())
            .map(output::Json)
            .map(output::status::Created),

        path!(@get / i32 /)
            .and(acquire_conn.clone())
            .and_then(|id, conn| {
                crate::api::find_post(id, conn)
                    .from_err()
                    .and_then(|conn_opt| conn_opt.ok_or_else(|| finchers::error::err_msg(StatusCode::NOT_FOUND, "not found"))
                    )
            })
            .map(output::Json),
    });

    finchers::launch(endpoint).start("127.0.0.1:4000");
    Ok(())
}
