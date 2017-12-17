use std::sync::Arc;


#[derive(Debug)]
pub struct OneshotFn<F1, F2> {
    inner: Inner<F1, F2>,
}

#[derive(Debug)]
enum Inner<F1, F2> {
    Owned(F1),
    Shared(Arc<F2>),
}
use self::Inner::*;

pub fn owned<F1, F2>(f: F1) -> OneshotFn<F1, F2> {
    OneshotFn { inner: Owned(f) }
}

pub fn shared<F1, F2>(f: Arc<F2>) -> OneshotFn<F1, F2> {
    OneshotFn { inner: Shared(f) }
}


pub trait Caller<T, R> {
    fn call(self, T) -> R;
}

impl<F1, F2, T, R> Caller<T, R> for OneshotFn<F1, F2>
where
    F1: FnOnce(T) -> R,
    F2: Fn(T) -> R,
{
    fn call(self, arg: T) -> R {
        match self.inner {
            Owned(f) => f(arg),
            Shared(f) => (*f)(arg),
        }
    }
}
