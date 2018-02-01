use std::collections::HashSet;
use std::io;
use std::mem;
use std::net::{SocketAddr, ToSocketAddrs};
use super::backend::DefaultBackend;

/// TCP level configuration
#[derive(Debug)]
pub struct Tcp<B = DefaultBackend> {
    pub(super) addrs: Vec<SocketAddr>,
    pub(super) backend: B,
}

impl Default for Tcp<DefaultBackend> {
    fn default() -> Self {
        Tcp {
            addrs: vec![],
            backend: Default::default(),
        }
    }
}

impl<B> Tcp<B> {
    /// Create a new instance of `Tcp` with given backend
    pub fn new(backend: B) -> Self {
        Tcp {
            backend,
            addrs: vec![],
        }
    }

    /// Set the listener addresses.
    pub fn set_addrs<S>(&mut self, addrs: S) -> io::Result<()>
    where
        S: ToSocketAddrs,
    {
        self.addrs = addrs.to_socket_addrs()?.collect();
        Ok(())
    }

    /// Returns the mutable reference of the inner backend
    pub fn backend(&mut self) -> &mut B {
        &mut self.backend
    }

    pub(super) fn normalize_addrs(&mut self) {
        if self.addrs.is_empty() {
            self.addrs.push("0.0.0.0:4000".parse().unwrap());
        } else {
            let set: HashSet<_> = mem::replace(&mut self.addrs, vec![]).into_iter().collect();
            self.addrs = set.into_iter().collect();
        }
    }
}
