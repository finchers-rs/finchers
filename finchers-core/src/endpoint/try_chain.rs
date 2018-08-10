// The original implementation is futures-util-preview/try_future/try_chain.rs.

use std::mem::PinMut;
use std::task;
use std::task::Poll;

use futures_core::future::TryFuture;

#[derive(Debug)]
pub(super) enum TryChain<F1, F2, T> {
    First(F1, Option<T>),
    Second(F2),
    Empty,
}

pub(super) enum TryChainAction<F2>
where
    F2: TryFuture,
{
    Future(F2),
    Output(Result<F2::Ok, F2::Error>),
}

impl<F1, F2, T> TryChain<F1, F2, T>
where
    F1: TryFuture,
    F2: TryFuture,
{
    pub(super) fn new(f1: F1, data: T) -> TryChain<F1, F2, T> {
        TryChain::First(f1, Some(data))
    }

    pub(super) fn poll<F>(
        self: PinMut<Self>,
        cx: &mut task::Context,
        f: F,
    ) -> Poll<Result<F2::Ok, F2::Error>>
    where
        F: FnOnce(Result<F1::Ok, F1::Error>, T) -> TryChainAction<F2>,
    {
        let mut f = Some(f);

        // Safety: the futures does not move in this method.
        let this = unsafe { PinMut::get_mut_unchecked(self) };

        loop {
            let (out, data) = match this {
                TryChain::First(f1, data) => {
                    match unsafe { PinMut::new_unchecked(f1) }.try_poll(cx) {
                        Poll::Pending => return Poll::Pending,
                        Poll::Ready(out) => (out, data.take().unwrap()),
                    }
                }
                TryChain::Second(f2) => return unsafe { PinMut::new_unchecked(f2) }.try_poll(cx),
                TryChain::Empty => panic!("This future has already polled."),
            };

            let f = f.take().unwrap();
            match f(out, data) {
                TryChainAction::Future(f2) => {
                    *this = TryChain::Second(f2);
                    continue;
                }
                TryChainAction::Output(out) => {
                    *this = TryChain::Empty;
                    return Poll::Ready(out);
                }
            }
        }
    }
}
