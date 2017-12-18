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
mod then;

pub use futures::{Async, Poll};

pub use self::and_then::{and_then, and_then_shared, AndThen};
pub use self::body::{Body, BodyError};
pub use self::context::TaskContext;
pub use self::from_err::{from_err, FromErr};
pub use self::futures::{future, TaskFuture};
pub use self::inspect::{inspect, inspect_shared, Inspect};
pub use self::join::{join, Join, Join3, Join4, Join5, Join6, join3, join4, join5, join6};
pub use self::join_all::{join_all, JoinAll};
pub use self::lazy::{lazy, Lazy};
pub use self::map_err::{map_err, map_err_shared, MapErr};
pub use self::map::{map, map_shared, Map};
pub use self::or_else::{or_else, or_else_shared, OrElse};
pub use self::poll_fn::{poll_fn, PollFn};
pub use self::result::{err, ok, result, TaskResult};
pub use self::then::{then, then_shared, Then};


pub trait Task {
    type Item;
    type Error;

    fn poll(&mut self, ctx: &mut TaskContext) -> Poll<Self::Item, Self::Error>;

    fn and_then<F, R>(self, f: F) -> AndThen<Self, F, fn(Self::Item) -> R, R>
    where
        Self: Sized,
        F: FnOnce(Self::Item) -> R,
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

    fn inspect<A, F>(self, f: F) -> Inspect<Self, F, fn(&Self::Item)>
    where
        Self: Sized,
        F: FnOnce(&Self::Item),
    {
        inspect(self, f)
    }

    fn join<T>(self, t: T) -> Join<Self, T, Self::Error>
    where
        Self: Sized,
        T: Task<Error = Self::Error>,
    {
        join(self, t)
    }

    fn join3<T1, T2>(self, t1: T1, t2: T2) -> Join3<Self, T1, T2, Self::Error>
    where
        Self: Sized,
        T1: Task<Error = Self::Error>,
        T2: Task<Error = Self::Error>,
    {
        join3(self, t1, t2)
    }

    fn join4<T1, T2, T3>(self, t1: T1, t2: T2, t3: T3) -> Join4<Self, T1, T2, T3, Self::Error>
    where
        Self: Sized,
        T1: Task<Error = Self::Error>,
        T2: Task<Error = Self::Error>,
        T3: Task<Error = Self::Error>,
    {
        join4(self, t1, t2, t3)
    }

    fn join5<T1, T2, T3, T4>(self, t1: T1, t2: T2, t3: T3, t4: T4) -> Join5<Self, T1, T2, T3, T4, Self::Error>
    where
        Self: Sized,
        T1: Task<Error = Self::Error>,
        T2: Task<Error = Self::Error>,
        T3: Task<Error = Self::Error>,
        T4: Task<Error = Self::Error>,
    {
        join5(self, t1, t2, t3, t4)
    }

    fn join6<T1, T2, T3, T4, T5>(
        self,
        t1: T1,
        t2: T2,
        t3: T3,
        t4: T4,
        t5: T5,
    ) -> Join6<Self, T1, T2, T3, T4, T5, Self::Error>
    where
        Self: Sized,
        T1: Task<Error = Self::Error>,
        T2: Task<Error = Self::Error>,
        T3: Task<Error = Self::Error>,
        T4: Task<Error = Self::Error>,
        T5: Task<Error = Self::Error>,
    {
        join6(self, t1, t2, t3, t4, t5)
    }

    fn map<F, R>(self, f: F) -> Map<Self, F, fn(Self::Item) -> R>
    where
        Self: Sized,
        F: FnOnce(Self::Item) -> R,
    {
        map(self, f)
    }

    fn map_err<F, R>(self, f: F) -> MapErr<Self, F, fn(Self::Error) -> R>
    where
        Self: Sized,
        F: FnOnce(Self::Error) -> R,
    {
        map_err(self, f)
    }

    fn or_else<F, R>(self, f: F) -> OrElse<Self, F, fn(Self::Error) -> R, R>
    where
        Self: Sized,
        F: FnOnce(Self::Error) -> R,
        R: IntoTask<Item = Self::Item>,
    {
        or_else(self, f)
    }

    fn then<F, R>(self, f: F) -> Then<Self, F, fn(Result<Self::Item, Self::Error>) -> R, R>
    where
        Self: Sized,
        F: FnOnce(Result<Self::Item, Self::Error>) -> R,
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
