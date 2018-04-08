use std::rc::Rc;
use std::sync::Arc;

/// The abstruction of a function with a parameter.
///
/// This trait is similar to `FnOnce(T) -> R`. The difference is that
/// `Rc<T>` and `Arc<T>` are also the implementors of this trait,
/// in order to represent the "clonable" functions shared between request handlers.
pub trait Callable<T> {
    /// Type of value returned from `call`.
    type Output;

    /// Call this function with `arg` and return its result.
    fn call(self, arg: T) -> Self::Output;
}

impl<F, T, R> Callable<T> for F
where
    F: FnOnce(T) -> R,
{
    type Output = R;

    #[inline(always)]
    fn call(self, arg: T) -> Self::Output {
        (self)(arg)
    }
}

impl<F, T, R> Callable<T> for Rc<F>
where
    F: Fn(T) -> R,
{
    type Output = R;

    fn call(self, arg: T) -> Self::Output {
        (*self)(arg)
    }
}

impl<F, T, R> Callable<T> for Arc<F>
where
    F: Fn(T) -> R,
{
    type Output = R;

    fn call(self, arg: T) -> Self::Output {
        (*self)(arg)
    }
}
