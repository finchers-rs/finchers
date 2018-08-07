use super::maybe_done::MaybeDone;
use crate::endpoint::{Context, EndpointBase, IntoEndpoint};
use crate::poll::Poll;
use crate::task::Task;
use std::{fmt, mem};

/// Create an endpoint which evaluates the all endpoint in the given collection sequentially.
pub fn all<I>(iter: I) -> All<<I::Item as IntoEndpoint>::Endpoint>
where
    I: IntoIterator,
    I::Item: IntoEndpoint,
{
    All {
        inner: iter.into_iter().map(IntoEndpoint::into_endpoint).collect(),
    }
}

#[allow(missing_docs)]
#[derive(Clone, Debug)]
pub struct All<E> {
    inner: Vec<E>,
}

impl<E> EndpointBase for All<E>
where
    E: EndpointBase,
{
    type Output = Vec<E::Output>;
    type Task = AllTask<E::Task>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Task> {
        let mut elems = Vec::with_capacity(self.inner.len());
        for e in &self.inner {
            let f = e.apply(cx)?;
            elems.push(MaybeDone::Pending(f));
        }
        Some(AllTask { elems })
    }
}

#[allow(missing_docs)]
pub struct AllTask<T: Task> {
    elems: Vec<MaybeDone<T>>,
}

impl<T> fmt::Debug for AllTask<T>
where
    T: Task + fmt::Debug,
    T::Output: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("AllTask")
            .field("elems", &self.elems)
            .finish()
    }
}

impl<T> Task for AllTask<T>
where
    T: Task,
{
    type Output = Vec<T::Output>;

    fn poll_task(&mut self) -> Poll<Self::Output> {
        let mut all_done = true;
        for i in 0..self.elems.len() {
            match self.elems[i].poll_done() {
                done => all_done = all_done & done,
            }
        }
        if all_done {
            let elems: Vec<T::Output> = mem::replace(&mut self.elems, Vec::new())
                .into_iter()
                .map(|mut m| m.take_item())
                .collect();
            Poll::Ready(elems)
        } else {
            Poll::Pending
        }
    }
}
