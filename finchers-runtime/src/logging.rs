use slog::{Drain, Level, Logger};
use slog_async::Async;
use slog_term;

pub struct Logging {
    root: Logger,
}

impl Logging {
    pub fn new(level: Level) -> Logging {
        let drain = slog_term::term_full()
            .filter(move |record| record.level() <= level)
            .fuse();
        let async_drain = Async::new(drain).build().fuse();

        Logging {
            root: Logger::root(async_drain, o!()),
        }
    }

    pub fn root(&self) -> &Logger {
        &self.root
    }

    pub fn request(&self) -> Logger {
        self.root.new(o!())
    }
}
