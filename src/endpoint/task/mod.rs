//! `Task` layer

pub(crate) mod and_then;
pub(crate) mod body;
pub(crate) mod chain;
pub(crate) mod header;
pub(crate) mod join;
pub(crate) mod join_all;
pub(crate) mod map_err;
pub(crate) mod map;
pub(crate) mod or;
pub(crate) mod task;

pub use self::and_then::AndThen;
pub use self::body::Body;
pub use self::header::{Header, HeaderOpt};
pub use self::join::{Join, Join3, Join4, Join5};
pub use self::join_all::JoinAll;
pub use self::map_err::MapErr;
pub use self::map::Map;
pub use self::or::Or;
pub use self::task::Task;
