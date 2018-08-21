use std::cell::Cell;
use std::mem::PinMut;
use std::ptr::NonNull;

use super::Input;

thread_local!(static CX: Cell<Option<NonNull<Input>>> = Cell::new(None));

struct SetOnDrop(Option<NonNull<Input>>);

impl Drop for SetOnDrop {
    fn drop(&mut self) {
        CX.with(|cx| cx.set(self.0));
    }
}

pub(crate) fn with_set_cx<R>(current: PinMut<'_, Input>, f: impl FnOnce() -> R) -> R {
    CX.with(|cx| {
        let ptr = unsafe { PinMut::get_mut_unchecked(current) };
        cx.set(NonNull::new(ptr))
    });
    let _reset = SetOnDrop(None);
    f()
}

/// Acquires a pinned reference to `Input` from the task context and executes the provided
/// function using its value.
///
/// This function is usually used to access the value of `Input` within the `Future` returned
/// by the `Endpoint`.
///
/// # Panics
///
/// A panic will occur if you call this function inside the provided closure `f`, since the
/// reference to `Input` on the task context is invalidated while executing `f`.
pub fn with_get_cx<R>(f: impl FnOnce(PinMut<'_, Input>) -> R) -> R {
    let prev = CX.with(|cx| cx.replace(None));
    let _reset = SetOnDrop(prev);
    match prev {
        Some(mut ptr) => unsafe { f(PinMut::new_unchecked(ptr.as_mut())) },
        None => panic!("reference to Input is not set at the task context."),
    }
}
