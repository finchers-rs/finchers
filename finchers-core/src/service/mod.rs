//! A lancher of the HTTP services

#![allow(missing_docs)]

pub mod backend;
mod server;
mod service;

#[doc(inline)]
pub use self::backend::TcpBackend;
pub use self::server::Server;
pub use self::service::*;
