use std::cell::Cell;
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
pub fn with_set_cx<R>(current: &mut Input, f: impl FnOnce() -> R) -> R {
    CX.with(|cx| cx.set(NonNull::new(current)));
    let _reset = SetOnDrop(None);
    f()
}

#[allow(missing_docs)]
pub fn with_get_cx<R>(f: impl FnOnce(&mut Input) -> R) -> R {
    let prev = CX.with(|cx| cx.replace(None));
    let _reset = SetOnDrop(prev);
    match prev {
        Some(mut ptr) => unsafe { f(ptr.as_mut()) },
        None => panic!(""),
    }
}
