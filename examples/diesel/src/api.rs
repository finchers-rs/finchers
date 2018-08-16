use diesel::prelude::*;
use serde::Deserialize;

use finchers::error::{internal_server_error, Error};
use finchers::input::query::{FromQuery, QueryItems, Serde};
use finchers::output::status::Created;
use finchers::output::{Json, Responder};

use crate::database::Connection;
use crate::model::{NewPost, Post};
use crate::schema::posts;

#[derive(Debug, Deserialize)]
pub struct Query {
    count: i64,
}

impl Default for Query {
    fn default() -> Query {
        Query { count: 20 }
    }
}

impl FromQuery for Query {
    type Error = <Serde<Query> as FromQuery>::Error;

    fn from_query(items: QueryItems) -> Result<Self, Self::Error> {
        FromQuery::from_query(items).map(Serde::into_inner)
    }
}

pub async fn get_posts(query: Option<Query>, conn: Connection) -> Result<impl Responder, Error> {
    let query = query.unwrap_or_default();
    let post = await!(conn.execute(move |conn| {
        use crate::schema::posts::dsl::*;
        posts.limit(query.count).load::<Post>(conn.get())
    })).map_err(internal_server_error)?;

    Ok(Json(post))
}

pub async fn create_post(new_post: NewPost, conn: Connection) -> Result<impl Responder, Error> {
    let post = await!(conn.execute(move |conn| {
        diesel::insert_into(posts::table)
            .values(&new_post)
            .get_result::<Post>(conn.get())
    })).map_err(internal_server_error)?;

    Ok(Created(Json(post)))
}

pub async fn find_post(i: i32, conn: Connection) -> Result<impl Responder, Error> {
    let post_opt = await!(conn.execute(move |conn| {
        use crate::schema::posts::dsl::{id, posts};
        posts
            .filter(id.eq(i))
            .get_result::<Post>(conn.get())
            .optional()
    })).map_err(internal_server_error)?;

    Ok(post_opt.map(Json))
}
