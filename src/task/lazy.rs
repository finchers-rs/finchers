#![allow(missing_docs)]

use std::mem;
use context::Context;
use super::{IntoTask, Poll, Task};

pub fn lazy<F, R>(f: F) -> Lazy<F, R>
where
    F: FnOnce(&mut Context) -> R,
    R: IntoTask,
{
    Lazy {
        inner: Inner::First(f),
    }
}

#[derive(Debug)]
pub struct Lazy<F, R>
where
    F: FnOnce(&mut Context) -> R,
    R: IntoTask,
{
    inner: Inner<F, R::Task>,
}

#[derive(Debug)]
enum Inner<F, R> {
    First(F),
    Second(R),
    Done,
}
use self::Inner::*;

impl<F, R> Lazy<F, R>
where
    F: FnOnce(&mut Context) -> R,
    R: IntoTask,
{
    fn get(&mut self, ctx: &mut Context) -> &mut R::Task {
        match self.inner {
            First(..) => {}
            Second(ref mut t) => return t,
            Done => panic!(),
        }
        match mem::replace(&mut self.inner, Done) {
            First(f) => self.inner = Second(f(ctx).into_task()),
            _ => panic!(),
        }
        match self.inner {
            Second(ref mut f) => f,
            _ => panic!(),
        }
    }
}

impl<F, R> Task for Lazy<F, R>
where
    F: FnOnce(&mut Context) -> R,
    R: IntoTask,
{
    type Item = R::Item;
    type Error = R::Error;

    fn poll(&mut self, ctx: &mut Context) -> Poll<Self::Item, Self::Error> {
        self.get(ctx).poll(ctx)
    }
}
