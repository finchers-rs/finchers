#![feature(
    pin,
    arbitrary_self_types,
    async_await,
    await_macro,
    futures_api
)]

#[macro_use]
extern crate diesel;

mod schema;

use failure::Fallible;
use serde::Deserialize;
use std::env;

use finchers::endpoint::EndpointExt;
use finchers::endpoints::body;
use finchers::endpoints::query::query;
use finchers::{route, routes};

use crate::database::ConnectionPool;

fn main() -> Fallible<()> {
    dotenv::dotenv()?;
    let pool = ConnectionPool::init(env::var("DATABASE_URL")?)?;

    let endpoint = route!(/"api"/"v1"/"posts").and(routes!{
        route!(@get /)
            .and(query())
            .and(pool.clone())
            .and_then(crate::api::get_posts),

        route!(@post /)
            .and(body::json())
            .and(pool.clone())
            .and_then(crate::api::create_post),

        route!(@get / i32 /)
            .and(pool.clone())
            .and_then(crate::api::find_post),
    });

    finchers::rt::launch(endpoint)?;
    Ok(())
}

mod api {
    use diesel::prelude::*;
    use serde::Deserialize;

    use finchers::error::Error;
    use finchers::input::query::{FromQuery, QueryItems, Serde};
    use finchers::output::status::Created;
    use finchers::output::Json;

    use crate::database::Connection;
    use crate::model::{NewPost, Post};

    #[derive(Debug, Deserialize)]
    pub struct Query {
        #[serde(default)]
        count: i64,
    }

    impl FromQuery for Query {
        type Error = <Serde<Query> as FromQuery>::Error;

        fn from_query(items: QueryItems) -> Result<Self, Self::Error> {
            FromQuery::from_query(items).map(Serde::into_inner)
        }
    }

    pub async fn get_posts(query: Query, conn: Connection) -> Result<Json<Vec<Post>>, Error> {
        let post = await!(conn.execute(move |conn| {
            use crate::schema::posts::dsl::*;
            posts.limit(query.count).load(conn.get())
        }))?;

        Ok(Json(post))
    }

    pub async fn create_post(
        new_post: NewPost,
        conn: Connection,
    ) -> Result<Created<Json<Post>>, Error> {
        let post = await!(conn.execute(move |conn| {
            use crate::schema::posts;
            use diesel::prelude::*;
            ::diesel::insert_into(posts::table)
                .values(&new_post)
                .get_result::<Post>(conn.get())
        }))?;
        Ok(Created(Json(post)))
    }

    pub async fn find_post(id: i32, conn: Connection) -> Result<Option<Json<Post>>, Error> {
        let post_opt = await!(conn.execute(move |conn| {
            use crate::schema::posts::dsl;
            use diesel::prelude::*;
            dsl::posts
                .filter(dsl::id.eq(id))
                .get_result::<Post>(conn.get())
                .optional()
        }))?;

        Ok(post_opt.map(Json))
    }
}

mod model {
    use crate::schema::posts;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Queryable, Serialize)]
    pub struct Post {
        pub id: i32,
        pub title: String,
        pub body: String,
        pub published: bool,
    }

    #[derive(Debug, Insertable, Deserialize)]
    #[table_name = "posts"]
    pub struct NewPost {
        pub title: String,
        pub body: String,
    }
}

mod database {
    use std::future::Future;
    use std::marker::PhantomData;
    use std::marker::Unpin;
    use std::mem::PinMut;
    use std::task::{self, Poll};

    use diesel::pg::PgConnection;
    use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
    use failure::{self, Fallible};
    use tokio::prelude::Async;
    use tokio_threadpool::blocking;

    use finchers::endpoint::Endpoint;
    use finchers::error::{internal_server_error, Error};
    use finchers::input::{Cursor, Input};

    #[derive(Clone)]
    pub struct ConnectionPool {
        pool: Pool<ConnectionManager<PgConnection>>,
    }

    impl ConnectionPool {
        pub fn init(url: impl Into<String>) -> Fallible<Self> {
            let manager = ConnectionManager::<PgConnection>::new(url.into());
            let pool = Pool::builder().max_size(15).build(manager)?;
            Ok(ConnectionPool { pool })
        }
    }

    impl Endpoint for ConnectionPool {
        type Output = (Connection,);
        type Future = AcquireConnection;

        fn apply<'c>(&self, _: PinMut<Input>, c: Cursor<'c>) -> Option<(Self::Future, Cursor<'c>)> {
            Some((
                AcquireConnection {
                    pool: self.pool.clone(),
                },
                c,
            ))
        }
    }

    pub struct AcquireConnection {
        pool: Pool<ConnectionManager<PgConnection>>,
    }

    impl Future for AcquireConnection {
        type Output = Result<(Connection,), Error>;

        fn poll(self: PinMut<Self>, _: &mut task::Context) -> Poll<Self::Output> {
            match blocking(|| self.pool.get()) {
                Ok(Async::NotReady) => Poll::Pending,
                Ok(Async::Ready(Ok(conn))) => Poll::Ready(Ok((Connection { conn },))),
                Ok(Async::Ready(Err(err))) => Poll::Ready(Err(internal_server_error(err))),
                Err(err) => Poll::Ready(Err(internal_server_error(err))),
            }
        }
    }

    pub struct Connection {
        conn: PooledConnection<ConnectionManager<PgConnection>>,
    }

    impl Connection {
        pub fn get(&self) -> &PgConnection {
            &*self.conn
        }

        pub fn execute<F, T, E>(self, f: F) -> Execute<F, T, E>
        where
            F: FnOnce(&Connection) -> Result<T, E> + Unpin,
            E: Into<failure::Error>,
        {
            Execute {
                conn: self,
                f_opt: Some(f),
                _marker: PhantomData,
            }
        }
    }

    pub struct Execute<F, T, E> {
        conn: Connection,
        f_opt: Option<F>,
        _marker: PhantomData<fn() -> Result<T, E>>,
    }

    impl<F, T, E> Future for Execute<F, T, E>
    where
        F: FnOnce(&Connection) -> Result<T, E> + Unpin,
        E: Into<failure::Error>,
    {
        type Output = Result<T, Error>;

        fn poll(self: PinMut<Self>, _: &mut task::Context) -> Poll<Self::Output> {
            let this = unsafe { PinMut::get_mut_unchecked(self) };
            match blocking(|| (this.f_opt.take().unwrap())(&this.conn)) {
                Ok(Async::NotReady) => Poll::Pending,
                Ok(Async::Ready(Ok(ok))) => Poll::Ready(Ok(ok)),
                Ok(Async::Ready(Err(err))) => Poll::Ready(Err(internal_server_error(err))),
                Err(err) => Poll::Ready(Err(internal_server_error(err))),
            }
        }
    }
}
