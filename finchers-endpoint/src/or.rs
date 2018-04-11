use either::Either;
use finchers_core::endpoint::{Context, Endpoint, IntoEndpoint};

pub fn new<E1, E2>(e1: E1, e2: E2) -> Or<E1::Endpoint, E2::Endpoint>
where
    E1: IntoEndpoint,
    E2: IntoEndpoint<Item = E1::Item>,
{
    Or {
        e1: e1.into_endpoint(),
        e2: e2.into_endpoint(),
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Or<E1, E2> {
    e1: E1,
    e2: E2,
}

impl<E1, E2> Endpoint for Or<E1, E2>
where
    E1: Endpoint,
    E2: Endpoint<Item = E1::Item>,
{
    type Item = E1::Item;
    type Task = Either<E1::Task, E2::Task>;

    fn apply(&self, cx2: &mut Context) -> Option<Self::Task> {
        let mut cx1 = cx2.clone();
        let t1 = self.e1.apply(&mut cx1);
        let t2 = self.e2.apply(cx2);
        match (t1, t2) {
            (Some(t1), Some(t2)) => {
                // If both endpoints are matched, the one with the larger number of
                // (consumed) path segments is choosen.
                let res = if cx1.segments().popped() > cx2.segments().popped() {
                    *cx2 = cx1;
                    Either::Left(t1)
                } else {
                    Either::Right(t2)
                };
                Some(res)
            }
            (Some(t1), None) => {
                *cx2 = cx1;
                Some(Either::Left(t1))
            }
            (None, Some(t2)) => Some(Either::Right(t2)),
            (None, None) => None,
        }
    }
}
