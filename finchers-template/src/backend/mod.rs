#![allow(missing_docs)]

pub(crate) mod askama;
pub(crate) mod engine;
pub(crate) mod handlebars;
pub(crate) mod horrorshow;
pub(crate) mod tera;

pub use self::engine::Engine;

#[cfg(feature = "use-askama")]
pub use self::askama::AskamaEngine;

#[cfg(feature = "use-handlebars")]
pub use self::handlebars::{AsHandlebars, HandlebarsEngine};

#[cfg(feature = "use-horrorshow")]
pub use self::horrorshow::HorrorshowEngine;

#[cfg(feature = "use-tera")]
pub use self::tera::{AsTera, TeraEngine};
