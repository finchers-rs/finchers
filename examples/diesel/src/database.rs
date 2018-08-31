use std::marker::{PhantomData, Unpin};
use std::pin::PinMut;

use std::future::Future;
use std::task;
use std::task::Poll;

use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use failure;
use failure::Fallible;
use tokio::prelude::Async;
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
    pub fn acquire_conn(&self) -> impl Future<Output = Fallible<Connection>> {
        AcquireConnection {
            pool: self.pool.clone(),
        }
    }
}

struct AcquireConnection {
    pool: Pool<ConnectionManager<PgConnection>>,
}

impl Future for AcquireConnection {
    type Output = Fallible<Connection>;

    fn poll(self: PinMut<Self>, _: &mut task::Context) -> Poll<Self::Output> {
        match blocking(|| self.pool.get()) {
            Ok(Async::NotReady) => Poll::Pending,
            Ok(Async::Ready(Ok(conn))) => Poll::Ready(Ok(Connection { conn })),
            Ok(Async::Ready(Err(err))) => Poll::Ready(Err(err.into())),
            Err(err) => Poll::Ready(Err(err.into())),
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

    pub fn execute<F, T, E>(self, f: F) -> impl Future<Output = Fallible<T>>
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

struct Execute<F, T, E> {
    conn: Connection,
    f_opt: Option<F>,
    _marker: PhantomData<fn() -> Result<T, E>>,
}

impl<F, T, E> Future for Execute<F, T, E>
where
    F: FnOnce(&Connection) -> Result<T, E> + Unpin,
    E: Into<failure::Error>,
{
    type Output = Fallible<T>;

    fn poll(self: PinMut<Self>, _: &mut task::Context) -> Poll<Self::Output> {
        let this = unsafe { PinMut::get_mut_unchecked(self) };
        match blocking(|| (this.f_opt.take().unwrap())(&this.conn)) {
            Ok(Async::NotReady) => Poll::Pending,
            Ok(Async::Ready(Ok(ok))) => Poll::Ready(Ok(ok)),
            Ok(Async::Ready(Err(err))) => Poll::Ready(Err(err.into())),
            Err(err) => Poll::Ready(Err(err.into())),
        }
    }
}
