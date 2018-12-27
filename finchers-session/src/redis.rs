//! The session backend using Redis.
//!
//! # Example
//!
//! ```
//! #[macro_use]
//! extern crate finchers;
//! extern crate finchers_session;
//!
//! use finchers::prelude::*;
//! use finchers_session::Session;
//! use finchers_session::redis::{
//!     Client,
//!     RedisBackend,
//!     RedisSession,
//! };
//! use std::time::Duration;
//!
//! # fn main() {
//! # drop(|| {
//! let client = Client::open("redis://127.0.0.1/").unwrap();
//! let backend = RedisBackend::new(client)
//!     .key_prefix("my-app-name")
//!     .cookie_name("sid")
//!     .timeout(Duration::from_secs(60*3));
//!
//! let endpoint = path!(@get /)
//!     .and(backend)
//!     .and_then(|session: Session<RedisSession>| {
//!         session.with(|_session| {
//!             // ...
//! #           Ok("done")
//!         })
//!     });
//! # finchers::server::start(endpoint).serve("127.0.0.1:4000")
//! # });
//! # }
//! ```

extern crate cookie;
extern crate redis;

use finchers;
use finchers::endpoint::{ApplyContext, ApplyResult, Endpoint};
use finchers::error::Error;
use finchers::input::Input;

use std::borrow::Cow;
use std::fmt;
use std::mem;
use std::sync::Arc;
use std::time::Duration;

use self::redis::async::Connection;
#[doc(no_inline)]
pub use self::redis::Client;
use self::redis::RedisFuture;

use self::cookie::Cookie;
use futures::{Async, Future, Poll};
use uuid::Uuid;

use session::{RawSession, Session};

#[derive(Debug)]
struct RedisSessionConfig {
    key_prefix: String,
    cookie_name: Cow<'static, str>,
    timeout: Option<Duration>,
}

impl RedisSessionConfig {
    fn key_name(&self, id: &Uuid) -> String {
        format!("{}:{}", self.key_prefix, id)
    }

    fn get_session_id(&self, input: &mut Input) -> Result<Option<Uuid>, Error> {
        if let Some(cookie) = input.cookies()?.get(&self.cookie_name) {
            let session_id: Uuid = cookie
                .value()
                .parse()
                .map_err(finchers::error::bad_request)?;
            return Ok(Some(session_id));
        }
        Ok(None)
    }
}

/// The instance of `SessionBackend` which uses Redis.
#[derive(Debug, Clone)]
pub struct RedisBackend {
    client: Client,
    config: Arc<RedisSessionConfig>,
}

impl RedisBackend {
    /// Create a new `RedisSessionBackend` from the specified Redis client.
    pub fn new(client: Client) -> RedisBackend {
        RedisBackend {
            client,
            config: Arc::new(RedisSessionConfig {
                key_prefix: "finchers-sesssion".into(),
                cookie_name: "session-id".into(),
                timeout: None,
            }),
        }
    }

    fn config_mut(&mut self) -> &mut RedisSessionConfig {
        Arc::get_mut(&mut self.config).expect("The instance has already shared.")
    }

    /// Set the prefix string used in the key name when stores the session value
    /// to Redis.
    ///
    /// The default value is "finchers-session"
    pub fn key_prefix(mut self, prefix: impl Into<String>) -> RedisBackend {
        self.config_mut().key_prefix = prefix.into();
        self
    }

    /// Set the name of Cookie entry which stores the session id.
    ///
    /// The default value is "session-id"
    pub fn cookie_name(mut self, name: impl Into<Cow<'static, str>>) -> RedisBackend {
        self.config_mut().cookie_name = name.into();
        self
    }

    /// Set the timeout of session value.
    pub fn timeout(mut self, timeout: Duration) -> RedisBackend {
        self.config_mut().timeout = Some(timeout);
        self
    }
}

impl<'a> Endpoint<'a> for RedisBackend {
    type Output = (Session<RedisSession>,);
    type Future = ReadFuture;

    fn apply(&self, cx: &mut ApplyContext<'_>) -> ApplyResult<Self::Future> {
        match self.config.get_session_id(cx.input()) {
            Ok(session_id) => Ok(ReadFuture::connecting(
                &self.client,
                &self.config,
                session_id,
            )),
            Err(err) => Ok(ReadFuture::failed(err)),
        }
    }
}

#[doc(hidden)]
#[allow(missing_debug_implementations)]
pub struct ReadFuture {
    state: ReadFutureState,
}

#[allow(missing_debug_implementations)]
enum ReadFutureState {
    Failed(Option<Error>),
    Connecting {
        future: RedisFuture<Connection>,
        config: Arc<RedisSessionConfig>,
        session_id: Option<Uuid>,
    },
    Fetch {
        future: RedisFuture<(Connection, Option<String>)>,
        config: Arc<RedisSessionConfig>,
        session_id: Uuid,
    },
    Done,
}

impl ReadFuture {
    fn failed(err: Error) -> ReadFuture {
        ReadFuture {
            state: ReadFutureState::Failed(Some(err)),
        }
    }

    fn connecting(
        client: &Client,
        config: &Arc<RedisSessionConfig>,
        session_id: Option<Uuid>,
    ) -> ReadFuture {
        ReadFuture {
            state: ReadFutureState::Connecting {
                future: client.get_async_connection(),
                config: config.clone(),
                session_id,
            },
        }
    }
}

impl Future for ReadFuture {
    type Item = (Session<RedisSession>,);
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        use self::ReadFutureState::*;
        loop {
            let (conn, value) = match self.state {
                Failed(ref mut err) => {
                    return Err(err.take().expect("This future has alread polled."))
                }
                Connecting { ref mut future, .. } => {
                    let conn = try_ready!(future.poll().map_err(finchers::error::fail));
                    (Some(conn), None)
                }
                Fetch { ref mut future, .. } => {
                    let (conn, value) = try_ready!(future.poll().map_err(finchers::error::fail));
                    (Some(conn), value)
                }
                Done => panic!("unexpected state"),
            };

            match (mem::replace(&mut self.state, Done), conn, value) {
                (
                    Connecting {
                        config,
                        session_id: Some(session_id),
                        ..
                    },
                    Some(conn),
                    None,
                ) => {
                    self.state = Fetch {
                        future: redis::cmd("GET")
                            .arg(config.key_name(&session_id))
                            .query_async(conn),
                        config,
                        session_id,
                    };
                }

                (
                    Fetch {
                        config, session_id, ..
                    },
                    Some(conn),
                    Some(value),
                ) => {
                    return Ok(Async::Ready((Session::new(RedisSession {
                        conn,
                        config,
                        session_id: Some(session_id),
                        value: Some(value),
                    }),)))
                }

                (
                    Connecting {
                        config,
                        session_id: None,
                        ..
                    },
                    Some(conn),
                    None,
                )
                | (Fetch { config, .. }, Some(conn), None) => {
                    return Ok(Async::Ready((Session::new(RedisSession {
                        conn,
                        config,
                        session_id: None,
                        value: None,
                    }),)));
                }

                _ => unreachable!("unexpected condition"),
            }
        }
    }
}

// ==== RedisSession ====

#[allow(missing_docs)]
pub struct RedisSession {
    conn: Connection,
    config: Arc<RedisSessionConfig>,
    session_id: Option<Uuid>,
    value: Option<String>,
}

impl fmt::Debug for RedisSession {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RedisSession")
            .field("config", &self.config)
            .field("session_id", &self.session_id)
            .field("value", &self.value)
            .finish()
    }
}

impl RawSession for RedisSession {
    type WriteFuture = WriteFuture;

    fn get(&self) -> Option<&str> {
        self.value.as_ref().map(|s| s.as_ref())
    }

    fn set(&mut self, value: String) {
        self.value = Some(value);
    }

    fn remove(&mut self) {
        self.value = None;
    }

    fn write(self, input: &mut Input) -> Self::WriteFuture {
        let Self {
            conn,
            config,
            session_id,
            value,
        } = self;

        match (session_id, value) {
            (Some(session_id), None) => {
                match input.cookies() {
                    Ok(jar) => jar.remove(Cookie::named(config.cookie_name.clone())),
                    Err(err) => return WriteFuture::failed(err),
                }
                let redis_key = config.key_name(&session_id);
                WriteFuture::cmd(conn, redis::cmd("DEL").arg(redis_key))
            }
            (session_id, Some(value)) => {
                let session_id = session_id.unwrap_or_else(Uuid::new_v4);
                match input.cookies() {
                    Ok(jar) => jar.add(Cookie::new(
                        config.cookie_name.clone(),
                        session_id.to_string(),
                    )),
                    Err(err) => return WriteFuture::failed(err),
                }
                let redis_key = config.key_name(&session_id);

                if let Some(timeout) = config.timeout {
                    WriteFuture::cmd(
                        conn,
                        redis::cmd("SETEX")
                            .arg(redis_key)
                            .arg(timeout.as_secs())
                            .arg(value),
                    )
                } else {
                    WriteFuture::cmd(conn, redis::cmd("SET").arg(redis_key).arg(value))
                }
            }
            (None, None) => WriteFuture::no_op(),
        }
    }
}

#[doc(hidden)]
#[allow(missing_debug_implementations)]
pub struct WriteFuture {
    state: WriteFutureState,
}

enum WriteFutureState {
    Noop,
    Failed(Option<Error>),
    Cmd(RedisFuture<(Connection, ())>),
}

impl WriteFuture {
    fn no_op() -> WriteFuture {
        WriteFuture {
            state: WriteFutureState::Noop,
        }
    }

    fn failed(err: Error) -> WriteFuture {
        WriteFuture {
            state: WriteFutureState::Failed(Some(err)),
        }
    }

    fn cmd(conn: Connection, cmd: &redis::Cmd) -> WriteFuture {
        WriteFuture {
            state: WriteFutureState::Cmd(cmd.query_async::<_, ()>(conn)),
        }
    }
}

impl Future for WriteFuture {
    type Item = ();
    type Error = Error;

    #[inline]
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        use self::WriteFutureState::*;
        match self.state {
            Noop => Ok(().into()),
            Failed(ref mut err) => Err(err.take().expect("The future has already polled.")),
            Cmd(ref mut future) => {
                let (_conn, ()) = try_ready!(future.poll().map_err(finchers::error::fail));
                Ok(Async::Ready(()))
            }
        }
    }
}
