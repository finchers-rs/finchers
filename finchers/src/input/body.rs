use std::error::Error as StdError;
use std::fmt;
use std::io;
use std::mem;

use bytes::Bytes;
use futures::Future;
use futures::Poll;
use http::header::HeaderMap;
use hyper::body::Body;

type Task = Box<dyn Future<Item = (), Error = ()> + Send + 'static>;

enum ReqBodyState {
    Unused(Body),
    Gone,
    Upgraded(Task),
}

impl fmt::Debug for ReqBodyState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReqBodyState::Unused(ref bd) => f.debug_tuple("Unused").field(bd).finish(),
            ReqBodyState::Gone => f.debug_tuple("Gone").finish(),
            ReqBodyState::Upgraded(..) => f.debug_tuple("Upgraded").finish(),
        }
    }
}

/// A type holding the instance of request body.
#[derive(Debug)]
pub struct ReqBody {
    state: ReqBodyState,
}

impl ReqBody {
    pub(crate) fn new(body: Body) -> ReqBody {
        ReqBody {
            state: ReqBodyState::Unused(body),
        }
    }

    /// Takes the instance of raw request body from the current context.
    ///
    /// It returns a `None` if the instance has already taken or the request
    /// has already upgraded to another protocol.
    pub fn take(&mut self) -> Option<Payload> {
        match mem::replace(&mut self.state, ReqBodyState::Gone) {
            ReqBodyState::Unused(body) => Some(Payload(body)),
            ReqBodyState::Gone => None,
            ReqBodyState::Upgraded(task) => {
                self.state = ReqBodyState::Upgraded(task);
                None
            }
        }
    }

    /// Returns whether the instance of `Payload` is available or not.
    ///
    /// It returns `false` if the request body has already taken or upgraded to another protocol.
    pub fn is_available(&self) -> bool {
        match self.state {
            ReqBodyState::Unused(..) => false,
            _ => true,
        }
    }

    /// Returns whether the protocol has already upgraded to another protocol.
    pub fn is_upgraded(&self) -> bool {
        match self.state {
            ReqBodyState::Upgraded(..) => true,
            _ => false,
        }
    }
}

/// An asynchronous stream of bytes representing the raw request body.
///
/// This type is currently a thin wrapper of `hyper::Body`.
#[derive(Debug)]
pub struct Payload(Body);

impl Payload {
    pub(crate) fn into_inner(self) -> Body {
        self.0
    }
}

impl ::hyper::body::Payload for Payload {
    type Data = io::Cursor<Bytes>;
    type Error = Box<dyn StdError + Send + Sync + 'static>;

    #[inline]
    fn poll_data(&mut self) -> Poll<Option<Self::Data>, Self::Error> {
        self.0
            .poll_data()
            .map(|x| x.map(|chunk_opt| chunk_opt.map(|chunk| io::Cursor::new(chunk.into_bytes()))))
            .map_err(Into::into)
    }

    #[inline]
    fn poll_trailers(&mut self) -> Poll<Option<HeaderMap>, Self::Error> {
        self.0.poll_trailers().map_err(Into::into)
    }

    #[inline]
    fn is_end_stream(&self) -> bool {
        self.0.is_end_stream()
    }

    #[inline]
    fn content_length(&self) -> Option<u64> {
        self.0.content_length()
    }
}

impl ::futures::Stream for Payload {
    type Item = Bytes;
    type Error = Box<dyn StdError + Send + Sync + 'static>;

    #[inline]
    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        self.0
            .poll()
            .map(|x| x.map(|chunk_opt| chunk_opt.map(Into::into)))
            .map_err(Into::into)
    }
}

mod rt {
    use super::{ReqBody, ReqBodyState, Task};

    use futures::{Future, IntoFuture};
    use hyper::body::Payload;
    use hyper::upgrade::Upgraded;
    use std::mem;

    impl ReqBody {
        pub(crate) fn upgrade<F, R>(&mut self, f: F)
        where
            F: FnOnce(Upgraded) -> R + Send + 'static,
            R: IntoFuture<Item = (), Error = ()>,
            R::Future: Send + 'static,
        {
            match mem::replace(&mut self.state, ReqBodyState::Gone) {
                ReqBodyState::Unused(body) => {
                    self.state = ReqBodyState::Upgraded(Box::new(
                        body.on_upgrade()
                            .map_err(|e| log::error!("during upgrading the protocol: {}", e))
                            .and_then(|upgraded| f(upgraded).into_future()),
                    ));
                }
                ReqBodyState::Gone => {}
                ReqBodyState::Upgraded(task) => {
                    self.state = ReqBodyState::Upgraded(task);
                }
            }
        }

        pub(crate) fn into_upgraded(self) -> Option<Task> {
            match self.state {
                ReqBodyState::Upgraded(task) => Some(task),
                _ => None,
            }
        }

        pub(crate) fn content_length(&self) -> Option<u64> {
            match self.state {
                ReqBodyState::Unused(ref body) => body.content_length(),
                _ => None,
            }
        }
    }
}

#[cfg(feature = "tower-web")]
mod payload_buf_stream {
    use super::Payload;

    use bytes::Bytes;
    use futures::Poll;
    use hyper::body::Payload as _HyperPayload;
    use std::error::Error as StdError;
    use std::io;

    use tower_web::util::buf_stream::size_hint::Builder;
    use tower_web::util::buf_stream::{BufStream, SizeHint};

    impl BufStream for Payload {
        type Item = io::Cursor<Bytes>;
        type Error = Box<dyn StdError + Send + Sync + 'static>;

        #[inline]
        fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
            self.poll_data()
        }

        fn size_hint(&self) -> SizeHint {
            let mut builder = Builder::new();
            if let Some(length) = self.content_length() {
                if length < usize::max_value() as u64 {
                    let length = length as usize;
                    builder.lower(length).upper(length);
                } else {
                    builder.lower(usize::max_value());
                }
            }
            builder.build()
        }
    }

}
