#![allow(missing_docs)]

mod and_then;
mod body;
mod chain;
mod context;
mod from_err;
mod futures;
mod inspect;
mod join;
mod join_all;
mod lazy;
mod map_err;
mod map;
mod maybe_done;
mod oneshot_fn;
mod or_else;
mod poll_fn;
mod result;
mod task;
mod then;

pub use futures::{Async, Poll};

pub use self::and_then::{and_then, AndThen};
pub use self::body::Body;
pub use self::context::TaskContext;
pub use self::from_err::{from_err, FromErr};
pub use self::futures::{future, TaskFuture};
pub use self::inspect::{inspect, Inspect};
pub use self::join::{join, Join, Join3, Join4, Join5, Join6, join3, join4, join5, join6};
pub use self::join_all::{join_all, JoinAll};
pub use self::lazy::{lazy, Lazy};
pub use self::map_err::{map_err, MapErr};
pub use self::map::{map, Map};
pub use self::or_else::{or_else, OrElse};
pub use self::poll_fn::{poll_fn, PollFn};
pub use self::result::{err, ok, result, TaskResult};
pub use self::task::{IntoTask, Task};
pub use self::then::{then, Then};


pub(crate) mod shared {
    pub use super::and_then::and_then_shared as and_then;
    pub use super::inspect::inspect_shared as inspect;
    pub use super::map_err::map_err_shared as map_err;
    pub use super::map::map_shared as map;
    pub use super::or_else::or_else_shared as or_else;
    pub use super::then::then_shared as then;
}
