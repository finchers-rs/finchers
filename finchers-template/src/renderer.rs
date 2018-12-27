use finchers::endpoint::wrapper::Wrapper;
use finchers::endpoint::{ApplyContext, ApplyResult, Endpoint};
use finchers::error;

use std::fmt;
use std::marker::PhantomData;

use futures::{Async, Future, Poll};
use http::header;
use http::header::HeaderValue;
use http::Response;
use mime::Mime;

use backend::engine::Engine;

lazy_static! {
    static ref DEFAULT_CONTENT_TYPE: HeaderValue =
        HeaderValue::from_static("text/html; charset=utf-8");
}

/// A struct which renders a context value to an HTTP response
/// using the specified template engine.
#[derive(Debug)]
pub struct Renderer<Eng> {
    engine: Eng,
    content_type: Option<HeaderValue>,
}

impl<Eng> Renderer<Eng> {
    /// Create a new `Renderer` from the specified engine.
    pub fn new(engine: Eng) -> Renderer<Eng> {
        Renderer {
            engine,
            content_type: None,
        }
    }

    /// Returns a reference to the inner template engine.
    pub fn engine(&self) -> &Eng {
        &self.engine
    }

    /// Returns a mutable reference to the inner template engine.
    pub fn engine_mut(&mut self) -> &mut Eng {
        &mut self.engine
    }

    /// Sets the value of content-type used in the rendered HTTP responses.
    pub fn content_type(mut self, value: &Mime) -> Renderer<Eng> {
        self.content_type = Some(
            value
                .as_ref()
                .parse()
                .expect("should be a valid header value"),
        );
        self
    }

    fn get_content_type<T>(&self, value: &T) -> HeaderValue
    where
        Eng: Engine<T>,
    {
        self.content_type
            .clone()
            .or_else(|| self.engine.content_type_hint(&value))
            .unwrap_or_else(|| DEFAULT_CONTENT_TYPE.clone())
    }

    fn render_response<T>(&self, value: T) -> error::Result<Response<Eng::Body>>
    where
        Eng: Engine<T>,
    {
        let content_type = self.get_content_type(&value);
        let body = self
            .engine
            .render(value)
            .map_err(|err| error::Error::from(err.into()))?;
        let mut response = Response::new(body);
        response
            .headers_mut()
            .insert(header::CONTENT_TYPE, content_type);
        Ok(response)
    }
}

impl<'a, E, Eng, T> Wrapper<'a, E> for Renderer<Eng>
where
    E: Endpoint<'a, Output = (T,)>,
    Eng: Engine<T> + 'a,
    T: 'a,
{
    type Output = (Response<Eng::Body>,);
    type Endpoint = RenderEndpoint<E, Eng, T>;

    fn wrap(self, endpoint: E) -> Self::Endpoint {
        RenderEndpoint {
            endpoint,
            renderer: self,
            _marker: PhantomData,
        }
    }
}

pub struct RenderEndpoint<E, Eng, T>
where
    Eng: Engine<T>,
{
    endpoint: E,
    renderer: Renderer<Eng>,
    _marker: PhantomData<fn() -> T>,
}

impl<E, Eng, T> fmt::Debug for RenderEndpoint<E, Eng, T>
where
    E: fmt::Debug,
    Eng: Engine<T> + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RenderEndpoint")
            .field("endpoint", &self.endpoint)
            .field("renderer", &self.renderer)
            .finish()
    }
}

impl<'a, E, Eng, T> Endpoint<'a> for RenderEndpoint<E, Eng, T>
where
    E: Endpoint<'a, Output = (T,)>,
    Eng: Engine<T> + 'a,
    T: 'a,
{
    type Output = (Response<Eng::Body>,);
    type Future = RenderFuture<'a, E, Eng, T>;

    fn apply(&'a self, cx: &mut ApplyContext<'_>) -> ApplyResult<Self::Future> {
        Ok(RenderFuture {
            future: self.endpoint.apply(cx)?,
            renderer: &self.renderer,
            _marker: PhantomData,
        })
    }
}

pub struct RenderFuture<'a, E: Endpoint<'a>, Eng: 'a, T: 'a>
where
    Eng: Engine<T>,
{
    future: E::Future,
    renderer: &'a Renderer<Eng>,
    _marker: PhantomData<fn() -> T>,
}

impl<'a, E, Eng, T> fmt::Debug for RenderFuture<'a, E, Eng, T>
where
    E: Endpoint<'a> + fmt::Debug,
    E::Future: fmt::Debug,
    Eng: Engine<T> + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RenderFuture")
            .field("future", &self.future)
            .field("renderer", &self.renderer)
            .finish()
    }
}

impl<'a, E, Eng, T> Future for RenderFuture<'a, E, Eng, T>
where
    E: Endpoint<'a, Output = (T,)>,
    Eng: Engine<T> + 'a,
{
    type Item = (Response<Eng::Body>,);
    type Error = error::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let (value,) = try_ready!(self.future.poll());
        self.renderer
            .render_response(value)
            .map(|response| Async::Ready((response,)))
    }
}

#[cfg(test)]
mod tests {
    use super::Renderer;
    use backend::engine::{Engine, EngineImpl};

    use finchers::error;
    use finchers::prelude::*;
    use finchers::test;
    use std::string::ToString;

    #[test]
    fn test_renderer() {
        struct DummyEngine;
        impl<T: ToString> Engine<T> for DummyEngine {}
        impl<T: ToString> EngineImpl<T> for DummyEngine {
            type Body = String;
            type Error = error::Never;
            fn render(&self, value: T) -> Result<Self::Body, Self::Error> {
                Ok(value.to_string())
            }
        }

        let mut runner = test::runner({
            endpoint::syntax::verb::get()
                .and(endpoint::syntax::param::<String>())
                .and(endpoint::syntax::eos())
                .wrap(Renderer::new(DummyEngine))
        });

        let response = runner.perform("/Amaterasu").unwrap();
        assert_eq!(response.status().as_u16(), 200);
        assert_matches!(
            response.headers().get("content-type"),
            Some(h) if h == "text/html; charset=utf-8"
        );
        assert_eq!(response.body().to_utf8().unwrap(), "Amaterasu");
    }

}
