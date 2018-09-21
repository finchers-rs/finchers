use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use failure::{Error, Fallible};
use futures::prelude::*;
use tokio_threadpool::blocking;

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

impl ConnectionPool {
    pub fn acquire_conn(&self) -> impl Future<Item = Connection, Error = Error> {
        let pool = self.pool.clone();
        futures::future::poll_fn(move || {
            let conn = futures::try_ready!(blocking(|| pool.get()))?;
            Ok(Connection { conn }.into())
        })
    }
}

pub struct Connection {
    conn: PooledConnection<ConnectionManager<PgConnection>>,
}

impl Connection {
    pub fn get(&self) -> &PgConnection {
        &*self.conn
    }

    pub fn execute<F, T, E>(self, f: F) -> impl Future<Item = T, Error = Error>
    where
        F: FnOnce(&Connection) -> Result<T, E>,
        E: Into<Error>,
    {
        let mut f_opt = Some(f);
        futures::future::poll_fn(move || {
            let x = futures::try_ready!(blocking(|| (f_opt.take().unwrap())(&self)))
                .map_err(Into::into)?;
            Ok(x.into())
        })
    }
}
