//! Definition of combinators

mod and_then;
mod from_err;
mod inspect;
mod join;
mod map;
mod map_err;
mod or;
mod or_else;
mod skip;
mod then;
mod with;

pub use self::and_then::{and_then, AndThen};
pub use self::from_err::{from_err, FromErr};
pub use self::inspect::{inspect, Inspect};
pub use self::join::{join, Join};
pub use self::map::{map, Map};
pub use self::map_err::{map_err, MapErr};
pub use self::or::{or, Or};
pub use self::or_else::{or_else, OrElse};
pub use self::skip::{skip, Skip};
pub use self::then::{then, Then};
pub use self::with::{with, With};
