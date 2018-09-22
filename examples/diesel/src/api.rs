use diesel::prelude::*;
use failure::Error;
use futures::prelude::*;
use serde::Deserialize;

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

pub fn get_posts(
    query: Option<Query>,
    conn: Connection,
) -> impl Future<Item = Vec<Post>, Error = Error> {
    let query = query.unwrap_or_default();
    conn.execute(move |conn| {
        use crate::schema::posts::dsl::*;
        posts.limit(query.count).load::<Post>(conn.get())
    })
}

pub fn create_post(new_post: NewPost, conn: Connection) -> impl Future<Item = Post, Error = Error> {
    conn.execute(move |conn| {
        diesel::insert_into(posts::table)
            .values(&new_post)
            .get_result::<Post>(conn.get())
    })
}

pub fn find_post(i: i32, conn: Connection) -> impl Future<Item = Option<Post>, Error = Error> {
    conn.execute(move |conn| {
        use crate::schema::posts::dsl::{id, posts};
        posts
            .filter(id.eq(i))
            .get_result::<Post>(conn.get())
            .optional()
    })
}
