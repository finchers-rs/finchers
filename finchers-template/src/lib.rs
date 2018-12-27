// FIXME: remove this feature gate as soon as the rustc version used in docs.rs is updated
#![cfg_attr(finchers_inject_extern_prelude, feature(extern_prelude))]

//! Template support for Finchers

#![doc(html_root_url = "https://docs.rs/finchers-template/0.2.0-dev")]
#![warn(
    missing_docs,
    missing_debug_implementations,
    nonstandard_style,
    rust_2018_idioms,
    unused,
)]
//#![warn(rust_2018_compatibility)]
#![cfg_attr(finchers_deny_warnings, deny(warnings))]
#![cfg_attr(finchers_deny_warnings, doc(test(attr(deny(warnings)))))]

extern crate failure;
extern crate finchers;
#[macro_use]
extern crate futures;
extern crate http;
#[macro_use]
extern crate lazy_static;
extern crate mime;
#[cfg(
    any(
        feature = "use-tera",
        feature = "use-handlebars",
        feature = "use-askama"
    )
)]
extern crate mime_guess;

#[cfg(any(feature = "use-tera", feature = "use-handlebars"))]
#[cfg_attr(
    all(test, any(feature = "use-handlebars", feature = "use-tera")),
    macro_use
)]
extern crate serde;

#[cfg(test)]
#[macro_use]
extern crate matches;

#[cfg(feature = "use-handlebars")]
extern crate handlebars;

#[cfg(feature = "use-tera")]
extern crate tera;

#[cfg(feature = "use-askama")]
extern crate askama;

#[cfg(feature = "use-horrorshow")]
#[cfg_attr(test, macro_use)]
extern crate horrorshow;

pub mod backend;
mod renderer;

pub use self::renderer::Renderer;

#[cfg(feature = "use-askama")]
pub use self::backend::askama::askama;

#[cfg(feature = "use-handlebars")]
pub use self::backend::handlebars::handlebars;

#[cfg(feature = "use-horrorshow")]
pub use self::backend::horrorshow::horrorshow;

#[cfg(feature = "use-tera")]
pub use self::backend::tera::tera;
