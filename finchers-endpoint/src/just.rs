#![allow(missing_docs)]

use finchers_core::Endpoint;
use finchers_core::endpoint::Context;
use finchers_core::outcome;

pub fn just<T>(x: T) -> Just<T>
where
    T: Clone + Send,
{
    Just { x }
}

#[derive(Debug, Clone, Copy)]
pub struct Just<T> {
    x: T,
}

impl<T> Endpoint for Just<T>
where
    T: Clone + Send,
{
    type Output = T;
    type Outcome = outcome::Ready<T>;

    fn apply(&self, _: &mut Context) -> Option<Self::Outcome> {
        Some(outcome::ready(self.x.clone()))
    }
}
