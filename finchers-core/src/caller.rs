use std::rc::Rc;
use std::sync::Arc;

pub trait Caller<T> {
    type Output;
    fn call(self, arg: T) -> Self::Output;
}

impl<F, T, R> Caller<T> for F
where
    F: FnOnce(T) -> R,
{
    type Output = R;

    fn call(self, arg: T) -> Self::Output {
        (self)(arg)
    }
}

impl<F, T, R> Caller<T> for Rc<F>
where
    F: Fn(T) -> R,
{
    type Output = R;

    fn call(self, arg: T) -> Self::Output {
        (*self)(arg)
    }
}

impl<F, T, R> Caller<T> for Arc<F>
where
    F: Fn(T) -> R,
{
    type Output = R;

    fn call(self, arg: T) -> Self::Output {
        (*self)(arg)
    }
}

pub struct BoxedCaller<T: ?Sized>(Box<T>);

impl<T: ?Sized> BoxedCaller<T> {
    pub fn new(caller: Box<T>) -> Self {
        BoxedCaller(caller)
    }
}

// TODO: FnBox?
impl<F, T, R> Caller<T> for BoxedCaller<F>
where
    F: FnMut(T) -> R,
{
    type Output = R;

    fn call(mut self, arg: T) -> Self::Output {
        (*self.0)(arg)
    }
}
