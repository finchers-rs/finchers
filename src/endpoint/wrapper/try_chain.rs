// The original implementation is futures-util-preview/try_future/try_chain.rs.

use futures::{Async, Future, Poll};

#[derive(Debug)]
pub(super) enum TryChain<F1, F2, T> {
    First(F1, Option<T>),
    Second(F2),
    Empty,
}

pub(super) enum TryChainAction<F2>
where
    F2: Future,
{
    Future(F2),
    Output(Result<F2::Item, F2::Error>),
}

impl<F1, F2, T> TryChain<F1, F2, T>
where
    F1: Future,
    F2: Future,
{
    pub(super) fn new(f1: F1, data: T) -> TryChain<F1, F2, T> {
        TryChain::First(f1, Some(data))
    }

    pub(super) fn poll<F>(&mut self, f: F) -> Poll<F2::Item, F2::Error>
    where
        F: FnOnce(Result<F1::Item, F1::Error>, T) -> TryChainAction<F2>,
    {
        let mut f = Some(f);
        loop {
            let (out, data) = match self {
                TryChain::First(ref mut f1, ref mut data) => match f1.poll() {
                    Ok(Async::NotReady) => return Ok(Async::NotReady),
                    Ok(Async::Ready(ok)) => (Ok(ok), data.take().unwrap()),
                    Err(err) => (Err(err), data.take().unwrap()),
                },
                TryChain::Second(ref mut f2) => return f2.poll(),
                TryChain::Empty => panic!("This future has already polled."),
            };

            let f = f.take().unwrap();
            match f(out, data) {
                TryChainAction::Future(f2) => {
                    *self = TryChain::Second(f2);
                    continue;
                }
                TryChainAction::Output(out) => {
                    *self = TryChain::Empty;
                    return out.map(Async::Ready);
                }
            }
        }
    }
}
