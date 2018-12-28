#![allow(proc_macro_derive_resolution_fallback)]

#[macro_use]
extern crate diesel;

mod model;
mod schema;

use failure::Fallible;
use http::StatusCode;

use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;

use finchers::prelude::*;
use finchers::rt::blocking_section;
use finchers::{output, path, routes};

use crate::model::{NewPost, Post};

type Conn = r2d2::PooledConnection<ConnectionManager<PgConnection>>;

fn main() -> Fallible<()> {
    dotenv::dotenv()?;

    let acquire_conn = {
        use std::env;
        use std::sync::Arc;

        let manager = ConnectionManager::<PgConnection>::new(env::var("DATABASE_URL")?);
        let pool = r2d2::Pool::builder().build(manager)?;
        Arc::new(endpoint::unit().and_then(move || {
            let pool = pool.clone();
            blocking_section(move || pool.get().map_err(finchers::error::fail))
        }))
    };

    let get_posts = path!(@get /)
        .and(endpoints::query::optional())
        .and(acquire_conn.clone())
        .and_then({
            #[derive(Debug, serde::Deserialize)]
            pub struct Query {
                count: i64,
            }

            |query: Option<Query>, conn: Conn| {
                let query = query.unwrap_or_else(|| Query { count: 20 });
                blocking_section(move || {
                    use crate::schema::posts::dsl::*;
                    use diesel::prelude::*;
                    posts
                        .limit(query.count)
                        .load::<Post>(&*conn)
                        .map(output::Json)
                        .map_err(finchers::error::fail)
                })
            }
        });

    let create_post = path!(@post /)
        .and(endpoints::body::json())
        .and(acquire_conn.clone())
        .and_then(|new_post: NewPost, conn: Conn| {
            blocking_section(move || {
                use diesel::prelude::*;
                diesel::insert_into(crate::schema::posts::table)
                    .values(&new_post)
                    .get_result::<Post>(&*conn)
                    .map(output::Json)
                    .map(output::status::Created)
                    .map_err(finchers::error::fail)
            })
        });

    let find_post =
        path!(@get / i32 /)
            .and(acquire_conn.clone())
            .and_then(|id: i32, conn: Conn| {
                blocking_section(move || {
                    use crate::schema::posts::dsl;
                    use diesel::prelude::*;
                    dsl::posts
                        .filter(dsl::id.eq(id))
                        .get_result::<Post>(&*conn)
                        .optional()
                        .map_err(finchers::error::fail)
                        .and_then(|conn_opt| {
                            conn_opt.ok_or_else(|| {
                                finchers::error::err_msg(StatusCode::NOT_FOUND, "not found")
                            })
                        })
                        .map(output::Json)
                })
            });

    let endpoint = path!(/"api"/"v1"/"posts").and(routes! {
        get_posts,
        create_post,
        find_post,
    });

    finchers::server::start(endpoint).serve("127.0.0.1:4000")?;
    Ok(())
}
