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

#[doc(hidden)]
pub fn with_set_cx<R>(current: PinMut<'_, Input>, f: impl FnOnce() -> R) -> R {
    CX.with(|cx| {
        let ptr = unsafe { PinMut::get_mut_unchecked(current) };
        cx.set(NonNull::new(ptr))
    });
    let _reset = SetOnDrop(None);
    f()
}

#[allow(missing_docs)]
pub fn with_get_cx<R>(f: impl FnOnce(PinMut<'_, Input>) -> R) -> R {
    let prev = CX.with(|cx| cx.replace(None));
    let _reset = SetOnDrop(prev);
    match prev {
        Some(mut ptr) => unsafe { f(PinMut::new_unchecked(ptr.as_mut())) },
        None => panic!(""),
    }
}
