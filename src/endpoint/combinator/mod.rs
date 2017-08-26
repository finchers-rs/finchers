//! Definition of combinators

pub mod either;

mod and_then;
mod map;
mod map_err;
mod ok;
mod or;
mod or_else;
mod product;
mod skip;
mod then;
mod with;

pub use self::and_then::{and_then, AndThen};
pub use self::map::{map, Map};
pub use self::map_err::{map_err, MapErr};
pub use self::ok::{ok, EndpointOk};
pub use self::or::{or, Or};
pub use self::or_else::{or_else, OrElse};
pub use self::skip::{skip, Skip};
pub use self::then::{then, Then};
pub use self::with::{with, With};
