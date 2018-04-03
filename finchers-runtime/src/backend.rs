//! The definitions of TCP backends

use std::io;
use std::net::SocketAddr;
use futures::Stream;
use tokio_core::net::{TcpListener, TcpStream};
use tokio_core::reactor::Handle;
use tokio_io::{AsyncRead, AsyncWrite};

pub fn default() -> DefaultBackend {
    DefaultBackend::default()
}

/// A TCP backend.
pub trait TcpBackend {
    /// The type of incoming streams.
    type Io: AsyncRead + AsyncWrite + 'static;

    /// A `Stream` returned from `incoming`
    type Incoming: Stream<Item = Self::Io, Error = io::Error> + 'static;

    /// Create a TCP listener and return a `Stream` of `Io`s.
    fn incoming(&self, addr: &SocketAddr, handle: &Handle) -> io::Result<Self::Incoming>;
}

/// The default backend
#[derive(Default, Debug)]
pub struct DefaultBackend {}

impl TcpBackend for DefaultBackend {
    type Io = ::tokio_core::net::TcpStream;
    type Incoming = ::futures::stream::Map<::tokio_core::net::Incoming, fn((TcpStream, SocketAddr)) -> TcpStream>;

    fn incoming(&self, addr: &SocketAddr, handle: &Handle) -> io::Result<Self::Incoming> {
        Ok(listener(addr, handle)?
            .incoming()
            .map((|(sock, _)| sock) as fn((TcpStream, SocketAddr)) -> TcpStream))
    }
}

#[cfg(feature = "tls")]
pub use self::tls::{tls, TlsBackend};

#[cfg(feature = "tls")]
mod tls {
    use super::*;
    use std::fmt;
    use futures::Future;
    use native_tls::{Pkcs12, TlsAcceptor};
    use tokio_tls::{TlsAcceptorExt, TlsStream};

    pub fn tls(pkcs12: Pkcs12) -> Result<TlsBackend, ::native_tls::Error> {
        TlsBackend::from_pkcs12(pkcs12)
    }

    /// The TCP backend with TLS support.
    pub struct TlsBackend {
        acceptor: TlsAcceptor,
    }

    impl fmt::Debug for TlsBackend {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.debug_struct("TlsBackend")
                .field("acceptor", &"[secret]")
                .finish()
        }
    }

    impl TlsBackend {
        /// Create a new instance of `TlsBackend` from given identity.
        pub fn from_pkcs12(pkcs12: Pkcs12) -> Result<Self, ::native_tls::Error> {
            let acceptor = TlsAcceptor::builder(pkcs12)?.build()?;
            Ok(TlsBackend { acceptor })
        }
    }

    impl From<TlsAcceptor> for TlsBackend {
        fn from(acceptor: TlsAcceptor) -> TlsBackend {
            TlsBackend { acceptor }
        }
    }

    impl TcpBackend for TlsBackend {
        type Io = TlsStream<TcpStream>;
        type Incoming = Box<Stream<Item = Self::Io, Error = io::Error>>;

        fn incoming(&self, addr: &SocketAddr, handle: &Handle) -> io::Result<Self::Incoming> {
            let acceptor = self.acceptor.clone();
            Ok(Box::new(listener(addr, handle)?.incoming().and_then(
                move |(sock, _)| {
                    acceptor
                        .accept_async(sock)
                        .map_err(|err| io::Error::new(io::ErrorKind::Other, format!("Accept error: {}", err)))
                },
            )))
        }
    }
}

fn listener(addr: &SocketAddr, handle: &Handle) -> io::Result<TcpListener> {
    use net2::TcpBuilder;
    let listener = match *addr {
        SocketAddr::V4(..) => TcpBuilder::new_v4()?,
        SocketAddr::V6(..) => TcpBuilder::new_v6()?,
    };
    listener.bind(addr)?;
    listener.reuse_address(true)?;
    #[cfg(unix)]
    {
        use net2::unix::UnixTcpBuilderExt;
        listener.reuse_port(true)?;
    }
    let l = listener.listen(128)?;
    TcpListener::from_listener(l, addr, handle)
}
