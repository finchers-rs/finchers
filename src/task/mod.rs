#![allow(missing_docs)]

mod and_then;
mod chain;
mod from_err;
mod futures;
mod inspect;
mod join;
mod join_all;
mod map_err;
mod map;
mod maybe_done;
mod or_else;
mod poll_fn;
mod result;
mod then;

pub use futures::{Async, Poll};

pub use self::and_then::{and_then, AndThen};
pub use self::from_err::{from_err, FromErr};
pub use self::futures::{future, TaskFuture};
pub use self::inspect::{inspect, Inspect};
pub use self::join::{join, Join, Join3, Join4, Join5, Join6, join3, join4, join5, join6};
pub use self::join_all::{join_all, JoinAll};
pub use self::map_err::{map_err, MapErr};
pub use self::map::{map, Map};
pub use self::or_else::{or_else, OrElse};
pub use self::poll_fn::{poll_fn, PollFn};
pub use self::result::{err, ok, result, TaskResult};
pub use self::then::{then, Then};

use std::sync::Arc;
use context::Context;


pub trait Task {
    type Item;
    type Error;

    fn poll(&mut self, ctx: &mut Context) -> Poll<Self::Item, Self::Error>;

    fn and_then<A, F, R>(self, f: A) -> AndThen<Self, F, R>
    where
        Self: Sized,
        A: Into<Arc<F>>,
        F: Fn(Self::Item) -> R,
        R: IntoTask<Error = Self::Error>,
    {
        and_then(self, f)
    }

    fn from_err<E>(self) -> FromErr<Self, E>
    where
        Self: Sized,
        E: From<Self::Error>,
    {
        from_err(self)
    }

    fn inspect<A, F>(self, f: A) -> Inspect<Self, F>
    where
        Self: Sized,
        A: Into<Arc<F>>,
        F: Fn(&Self::Item),
    {
        inspect(self, f)
    }

    // TODO: add join(), join3(), join4(), join5(), join6(),

    fn map<A, F, R>(self, f: A) -> Map<Self, F, R>
    where
        Self: Sized,
        A: Into<Arc<F>>,
        F: Fn(Self::Item) -> R,
    {
        map(self, f)
    }

    fn map_err<A, F, R>(self, f: A) -> MapErr<Self, F, R>
    where
        Self: Sized,
        A: Into<Arc<F>>,
        F: Fn(Self::Error) -> R,
    {
        map_err(self, f)
    }

    fn or_else<A, F, R>(self, f: A) -> OrElse<Self, F, R>
    where
        Self: Sized,
        A: Into<Arc<F>>,
        F: Fn(Self::Error) -> R,
        R: IntoTask<Item = Self::Item>,
    {
        or_else(self, f)
    }

    fn then<A, F, R>(self, f: A) -> Then<Self, F, R>
    where
        Self: Sized,
        A: Into<Arc<F>>,
        F: Fn(Result<Self::Item, Self::Error>) -> R,
        R: IntoTask,
    {
        then(self, f)
    }
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
