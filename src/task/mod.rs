#![allow(missing_docs)]

pub(crate) mod and_then;
pub(crate) mod body;
pub(crate) mod chain;
pub(crate) mod context;
pub(crate) mod from_err;
pub(crate) mod futures;
pub(crate) mod inspect;
pub(crate) mod join;
pub(crate) mod join_all;
pub(crate) mod lazy;
pub(crate) mod map_err;
pub(crate) mod map;
pub(crate) mod maybe_done;
pub(crate) mod or_else;
pub(crate) mod poll_fn;
pub(crate) mod result;
pub(crate) mod task;
pub(crate) mod then;

pub use futures::{Async, Poll};

pub use self::and_then::AndThen;
pub use self::body::Body;
pub use self::context::TaskContext;
pub use self::from_err::FromErr;
pub use self::futures::{future, TaskFuture};
pub use self::inspect::Inspect;
pub use self::join::{Join, Join3, Join4, Join5, Join6};
pub use self::join_all::{join_all, JoinAll};
pub use self::lazy::{lazy, Lazy};
pub use self::map_err::MapErr;
pub use self::map::Map;
pub use self::or_else::OrElse;
pub use self::poll_fn::{poll_fn, PollFn};
pub use self::result::{err, ok, result, TaskResult};
pub use self::task::{IntoTask, Task};
pub use self::then::Then;
