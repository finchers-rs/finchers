use super::Input;
use endpoint;

#[doc(hidden)]
#[deprecated(
    since = "0.12.0-alpha.9",
    note = "use `endpoint::with_get_cx()` instead."
)]
#[inline(always)]
pub fn with_get_cx<R>(f: impl FnOnce(&mut Input) -> R) -> R {
    endpoint::with_get_cx(|cx| f(cx.input()))
}
