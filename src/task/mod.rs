#![allow(missing_docs)]

mod and_then;
mod chain;
mod from_err;
mod futures;
mod inspect;
mod join;
mod map_err;
mod map;
mod maybe_done;
mod or_else;
mod or;
mod result;
mod then;

use context::Context;

pub use futures::{Async, Poll};

pub use self::and_then::{and_then, AndThen};
pub use self::from_err::{from_err, FromErr};
pub use self::futures::{future, TaskFuture};
pub use self::inspect::{inspect, Inspect};
pub use self::join::{join, Join};
pub use self::map_err::{map_err, MapErr};
pub use self::map::{map, Map};
pub use self::or_else::{or_else, OrElse};
pub use self::or::{left, right, Or};
pub use self::result::{err, ok, result, TaskResult};
pub use self::then::{then, Then};


pub trait Task {
    type Item;
    type Error;

    fn poll(&mut self, ctx: &mut Context) -> Poll<Self::Item, Self::Error>;
}


pub trait IntoTask {
    type Item;
    type Error;
    type Task: Task<Item = Self::Item, Error = Self::Error>;

    fn into_task(self) -> Self::Task;
}

impl<T: Task> IntoTask for T {
    type Item = T::Item;
    type Error = T::Error;
    type Task = T;
    fn into_task(self) -> Self::Task {
        self
    }
}

impl<T, E> IntoTask for Result<T, E> {
    type Item = T;
    type Error = E;
    type Task = TaskResult<T, E>;

    fn into_task(self) -> Self::Task {
        result(self)
    }
}
