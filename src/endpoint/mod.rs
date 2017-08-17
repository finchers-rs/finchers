pub mod param;
pub mod path;

use futures::{Future, IntoFuture};
use input::Input;


pub trait Endpoint {
    type Item;
    type Error;
    type Future: Future<Item = Self::Item, Error = Self::Error>;

    fn apply(&self, input: Input) -> Result<Self::Future, Input>;
}

impl<F, R> Endpoint for F
where
    F: Fn(Input) -> Result<R, Input>,
    R: IntoFuture,
{
    type Item = R::Item;
    type Error = R::Error;
    type Future = R::Future;

    fn apply(&self, input: Input) -> Result<Self::Future, Input> {
        (*self)(input).map(IntoFuture::into_future)
    }
}

#[cfg(test)]
mod tests {
    use hyper::Get;
    use input::Input;
    use super::Endpoint;
    use super::path::{path, path_end};
    use super::param::param;

    #[test]
    fn case1() {
        let endpoint = path("foo", path("bar", path_end(param("hello"))));
        let input = Input::new(Get, "/foo/bar?hello=world");
        let output = endpoint.apply(input);
        assert!(output.is_ok());
    }

    #[test]
    fn case1_1() {
        let endpoint = path("foo", path("bar", path_end(param("hello"))));
        let input = Input::new(Get, "/foo/bar/?hello=world");
        let output = endpoint.apply(input);
        assert!(output.is_ok());
    }

    #[test]
    fn case1_2() {
        let endpoint = path("foo", path("bar", path_end(param("hello"))));
        let input = Input::new(Get, "/foo/bar/");
        let output = endpoint.apply(input);
        assert!(output.is_err());
    }

    #[test]
    fn case2() {
        let endpoint = path("foo", path("bar", path_end(param("hello"))));
        let input = Input::new(Get, "/foo/bar/baz?hello=world");
        let output = endpoint.apply(input);
        assert!(output.is_err());
    }

    #[test]
    fn case3() {
        let endpoint = path("foo", path("bar", path_end(param("hello"))));
        let input = Input::new(Get, "/foo/?hello=world");
        let output = endpoint.apply(input);
        assert!(output.is_err());
    }
}
