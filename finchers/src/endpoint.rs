//! Components for constructing `Endpoint`.

mod boxed;
pub mod ext;
pub mod syntax;

// re-exports
pub use self::{
    boxed::{EndpointObj, LocalEndpointObj},
    ext::EndpointExt,
};

use {
    crate::{
        action::{
            EndpointAction, //
            Oneshot,
            OneshotAction,
            PreflightContext,
        },
        common::Tuple,
        error::Error,
    },
    std::{rc::Rc, sync::Arc},
};

/// A trait indicating that the type has an implementation of `Endpoint<Bd>`.
///
/// The purpose of this trait is to implement the extension methods to `Endpoint`s
/// in situation when the type of request body is unknown.
pub trait IsEndpoint {
    /// Converts this endpoint into an `EndpointObj`.
    fn boxed<Bd, T>(self) -> EndpointObj<Bd, T>
    where
        Self: Endpoint<Bd, Output = T> + Send + Sync + 'static + Sized,
        Self::Action: Send + 'static,
        T: Tuple,
    {
        EndpointObj::new(self)
    }

    /// Converts this endpoint into a `LocalEndpointObj`.
    fn boxed_local<Bd, T>(self) -> LocalEndpointObj<Bd, T>
    where
        Self: Endpoint<Bd, Output = T> + 'static + Sized,
        Self::Action: 'static,
        T: Tuple,
    {
        LocalEndpointObj::new(self)
    }
}

impl<'a, E: IsEndpoint + ?Sized> IsEndpoint for &'a E {}
impl<E: IsEndpoint + ?Sized> IsEndpoint for Box<E> {}
impl<E: IsEndpoint + ?Sized> IsEndpoint for Rc<E> {}
impl<E: IsEndpoint + ?Sized> IsEndpoint for Arc<E> {}

/// Trait representing an endpoint, the main trait for abstracting
/// HTTP services in Finchers.
///
/// The endpoint behaves as an *asynchronous* process that takes
/// an HTTP request and convert it into a value of the associated type.
/// The process that handles an request is abstracted by `EndpointAction`,
/// and that instances are constructed by `Endpoint` for each request.
pub trait Endpoint<Bd>: IsEndpoint {
    /// The type of value that will be returned from `Action`.
    type Output: Tuple;

    /// The type of `EndpointAction` associated with this endpoint.
    type Action: EndpointAction<Bd, Output = Self::Output>;

    /// Spawns an instance of `Action` that applies to an request.
    fn action(&self) -> Self::Action;

    /// Add an annotation that the associated type `Output` is fixed to `T`.
    #[inline(always)]
    fn with_output<T: Tuple>(self) -> Self
    where
        Self: Endpoint<Bd, Output = T> + Sized,
    {
        self
    }
}

impl<'a, E, Bd> Endpoint<Bd> for &'a E
where
    E: Endpoint<Bd>,
{
    type Output = E::Output;
    type Action = E::Action;

    fn action(&self) -> Self::Action {
        (**self).action()
    }
}

impl<E, Bd> Endpoint<Bd> for Box<E>
where
    E: Endpoint<Bd>,
{
    type Output = E::Output;
    type Action = E::Action;

    fn action(&self) -> Self::Action {
        (**self).action()
    }
}

impl<E, Bd> Endpoint<Bd> for Rc<E>
where
    E: Endpoint<Bd>,
{
    type Output = E::Output;
    type Action = E::Action;

    fn action(&self) -> Self::Action {
        (**self).action()
    }
}

impl<E, Bd> Endpoint<Bd> for Arc<E>
where
    E: Endpoint<Bd>,
{
    type Output = E::Output;
    type Action = E::Action;

    fn action(&self) -> Self::Action {
        (**self).action()
    }
}

/// Create an endpoint from the specified closure that returns an `EndpointAction`.
pub fn endpoint<Bd, R>(
    f: impl Fn() -> R,
) -> impl Endpoint<
    Bd, //
    Output = R::Output,
    Action = R,
>
where
    R: EndpointAction<Bd>,
{
    #[allow(missing_debug_implementations)]
    struct ApplyEndpoint<F>(F);

    impl<F> IsEndpoint for ApplyEndpoint<F> {}

    impl<F, Bd, R> Endpoint<Bd> for ApplyEndpoint<F>
    where
        F: Fn() -> R,
        R: EndpointAction<Bd>,
    {
        type Output = R::Output;
        type Action = R;

        fn action(&self) -> Self::Action {
            (self.0)()
        }
    }

    ApplyEndpoint(f)
}

/// Create an endpoint which simply returns an unit (`()`).
#[inline]
pub fn unit<Bd>() -> impl Endpoint<
    Bd,
    Output = (),
    Action = Oneshot<self::unit::UnitAction>, // private
> {
    endpoint(|| self::unit::UnitAction(()).into_action())
}

mod unit {
    use super::*;

    #[allow(missing_debug_implementations)]
    pub struct UnitAction(pub(super) ());

    impl OneshotAction for UnitAction {
        type Output = ();

        fn preflight(self, _: &mut PreflightContext<'_>) -> Result<Self::Output, Error> {
            Ok(())
        }
    }
}

/// Create an endpoint which simply clones the specified value.
///
/// # Examples
///
/// ```ignore
/// # #[macro_use]
/// # extern crate finchers;
/// # extern crate futures;
/// # use finchers::prelude::*;
/// # use finchers::endpoint::value;
/// #
/// #[derive(Clone)]
/// struct Conn {
///     // ...
/// #   _p: (),
/// }
///
/// # fn main() {
/// let conn = {
///     // do some stuff...
/// #   Conn { _p: () }
/// };
///
/// let endpoint = path!(@get / "posts" / u32 /)
///     .and(value(conn))
///     .and_then(|id: u32, conn: Conn| {
///         // ...
/// #       drop(id);
/// #       futures::future::ok::<_, finchers::error::Never>(conn)
///     });
/// # drop(endpoint);
/// # }
/// ```
#[inline]
pub fn value<Bd, T: Clone>(
    x: T,
) -> impl Endpoint<
    Bd,
    Output = (T,),
    Action = Oneshot<self::value::ValueAction<T>>, // private
> {
    endpoint(move || self::value::ValueAction { x: x.clone() }.into_action())
}

mod value {
    use super::*;

    // not a public API.
    #[derive(Debug)]
    pub struct ValueAction<T> {
        pub(super) x: T,
    }

    impl<T> OneshotAction for ValueAction<T> {
        type Output = (T,);

        fn preflight(self, _: &mut PreflightContext<'_>) -> Result<Self::Output, Error> {
            Ok((self.x,))
        }
    }
}

// ==== EndpointAction ====
