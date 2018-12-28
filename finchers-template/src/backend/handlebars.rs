#![cfg(feature = "use-handlebars")]

use super::engine::{Engine, EngineImpl};
use renderer::Renderer;

use failure::SyncFailure;
use handlebars::Handlebars;
use http::header::HeaderValue;
use mime_guess::guess_mime_type_opt;
use serde::Serialize;
use std::borrow::Cow;

pub trait AsHandlebars {
    fn as_handlebars(&self) -> &Handlebars;
}

impl AsHandlebars for Handlebars {
    fn as_handlebars(&self) -> &Handlebars {
        self
    }
}

impl<T: AsHandlebars> AsHandlebars for Box<T> {
    fn as_handlebars(&self) -> &Handlebars {
        (**self).as_handlebars()
    }
}

impl<T: AsHandlebars> AsHandlebars for ::std::rc::Rc<T> {
    fn as_handlebars(&self) -> &Handlebars {
        (**self).as_handlebars()
    }
}

impl<T: AsHandlebars> AsHandlebars for ::std::sync::Arc<T> {
    fn as_handlebars(&self) -> &Handlebars {
        (**self).as_handlebars()
    }
}

pub fn handlebars<H>(
    registry: H,
    name: impl Into<Cow<'static, str>>,
) -> Renderer<HandlebarsEngine<H>>
where
    H: AsHandlebars,
{
    Renderer::new(HandlebarsEngine::new(registry, name))
}

#[derive(Debug)]
pub struct HandlebarsEngine<H> {
    registry: H,
    name: Cow<'static, str>,
    content_type: Option<HeaderValue>,
}

impl<H> HandlebarsEngine<H>
where
    H: AsHandlebars,
{
    pub fn new(registry: H, name: impl Into<Cow<'static, str>>) -> HandlebarsEngine<H> {
        let name = name.into();
        let content_type = guess_mime_type_opt(&*name)
            .map(|s| s.as_ref().parse().expect("should be a valid header value"));
        HandlebarsEngine {
            registry,
            name,
            content_type,
        }
    }

    pub fn set_template_name(&mut self, name: impl Into<Cow<'static, str>>) {
        self.name = name.into();
        if let Some(value) = guess_mime_type_opt(&*self.name)
            .map(|s| s.as_ref().parse().expect("should be a valid header name"))
        {
            self.content_type = Some(value);
        }
    }
}

impl<H, T: Serialize> Engine<T> for HandlebarsEngine<H> where H: AsHandlebars {}

impl<H, CtxT: Serialize> EngineImpl<CtxT> for HandlebarsEngine<H>
where
    H: AsHandlebars,
{
    type Body = String;
    type Error = SyncFailure<::handlebars::RenderError>;

    fn content_type_hint(&self, _: &CtxT) -> Option<HeaderValue> {
        self.content_type.clone()
    }

    fn render(&self, value: CtxT) -> Result<Self::Body, Self::Error> {
        self.registry
            .as_handlebars()
            .render(&self.name, &value)
            .map_err(SyncFailure::new)
    }
}

#[test]
fn test_handlebars() {
    #[derive(Debug, Serialize)]
    struct Context {
        name: String,
    }

    let mut registry = Handlebars::new();
    registry
        .register_template_string("index.html", "{{ name }}")
        .unwrap();

    let value = Context {
        name: "Alice".into(),
    };

    let engine = HandlebarsEngine::new(registry, "index.html");
    let body = engine.render(value).unwrap();
    assert_eq!(body, "Alice");
}
