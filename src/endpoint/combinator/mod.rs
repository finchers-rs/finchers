//! Definition of combinators

pub mod either;

mod and_then;
mod map;
mod ok;
mod or;
mod product;
mod skip;
mod with;

pub use self::and_then::{and_then, AndThen};
pub use self::map::{map, Map};
pub use self::with::{with, With};
pub use self::ok::{ok, EndpointOk};
pub use self::skip::{skip, Skip};
pub use self::or::{or, Or};
