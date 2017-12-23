use super::*;


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
        and_then::and_then(self, f)
    }

    fn from_err<E>(self) -> FromErr<Self, E>
    where
        Self: Sized,
        E: From<Self::Error>,
    {
        from_err::from_err(self)
    }

    fn inspect<A, F>(self, f: F) -> Inspect<Self, F, fn(&Self::Item)>
    where
        Self: Sized,
        F: FnOnce(&Self::Item),
    {
        inspect::inspect(self, f)
    }

    fn join<T>(self, t: T) -> Join<Self, T, Self::Error>
    where
        Self: Sized,
        T: Task<Error = Self::Error>,
    {
        join::join(self, t)
    }

    fn map<F, R>(self, f: F) -> Map<Self, F, fn(Self::Item) -> R>
    where
        Self: Sized,
        F: FnOnce(Self::Item) -> R,
    {
        map::map(self, f)
    }

    fn map_err<F, R>(self, f: F) -> MapErr<Self, F, fn(Self::Error) -> R>
    where
        Self: Sized,
        F: FnOnce(Self::Error) -> R,
    {
        map_err::map_err(self, f)
    }

    fn or_else<F, R>(self, f: F) -> OrElse<Self, F, fn(Self::Error) -> R, R>
    where
        Self: Sized,
        F: FnOnce(Self::Error) -> R,
        R: IntoTask<Item = Self::Item>,
    {
        or_else::or_else(self, f)
    }

    fn then<F, R>(self, f: F) -> Then<Self, F, fn(Result<Self::Item, Self::Error>) -> R, R>
    where
        Self: Sized,
        F: FnOnce(Result<Self::Item, Self::Error>) -> R,
        R: IntoTask,
    {
        then::then(self, f)
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
