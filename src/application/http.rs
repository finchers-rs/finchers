use hyper::Chunk;
use hyper::server;

/// HTTP-level configuration
#[derive(Debug)]
pub struct Http(server::Http<Chunk>);

impl Default for Http {
    fn default() -> Self {
        Http(server::Http::new())
    }
}

impl Http {
    /// Enable or disable `Keep-alive` option
    pub fn keep_alive(&mut self, enabled: bool) -> &mut Self {
        self.0.keep_alive(enabled);
        self
    }

    /// Enable pipeline mode
    pub fn pipeline(&mut self, enabled: bool) -> &mut Self {
        self.0.pipeline(enabled);
        self
    }

    pub(super) fn inner(&self) -> &server::Http<Chunk> {
        &self.0
    }
}
