/// HTTP-level configuration
#[derive(Debug)]
pub struct Http {
    pub(super) pipeline: bool,
    pub(super) keep_alive: bool,
}

impl Default for Http {
    fn default() -> Self {
        Http {
            pipeline: true,
            keep_alive: false,
        }
    }
}

impl Http {
    /// Enable or disable `Keep-alive` option
    pub fn keep_alive(&mut self, enabled: bool) -> &mut Self {
        self.keep_alive = enabled;
        self
    }

    /// Enable pipeline mode
    pub fn pipeline(&mut self, enabled: bool) -> &mut Self {
        self.pipeline = enabled;
        self
    }
}
