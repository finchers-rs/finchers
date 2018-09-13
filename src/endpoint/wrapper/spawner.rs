use futures_core::task::Spawn;
use futures_util::future;
use futures_util::future::FutureExt;
use futures_util::try_future;
use futures_util::try_future::TryFutureExt;

use crate::endpoint::{Context, Endpoint, EndpointResult};

use super::Wrapper;

/// Create a `Wrapper` to construct endpoints whose `Future` uses the specified spawner.
pub fn spawner<Sp>(spawner: Sp) -> Spawner<Sp>
where
    Sp: Spawn + Clone,
{
    Spawner { spawner }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct Spawner<Sp> {
    spawner: Sp,
}

impl<'a, E, Sp> Wrapper<'a, E> for Spawner<Sp>
where
    E: Endpoint<'a>,
    Sp: Spawn + Clone + 'a,
{
    type Output = E::Output;
    type Endpoint = SpawnerEndpoint<E, Sp>;

    fn wrap(self, endpoint: E) -> Self::Endpoint {
        SpawnerEndpoint {
            endpoint,
            spawner: self.spawner,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct SpawnerEndpoint<E, Sp> {
    endpoint: E,
    spawner: Sp,
}

impl<'a, E, Sp> Endpoint<'a> for SpawnerEndpoint<E, Sp>
where
    E: Endpoint<'a>,
    Sp: Spawn + Clone + 'a,
{
    type Output = E::Output;
    type Future = future::WithSpawner<try_future::IntoFuture<E::Future>, Sp>;

    #[inline]
    fn apply(&'a self, cx: &mut Context<'_>) -> EndpointResult<Self::Future> {
        self.endpoint
            .apply(cx)
            .map(|future| future.into_future().with_spawner(self.spawner.clone()))
    }
}
