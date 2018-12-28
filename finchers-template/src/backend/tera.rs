#![cfg(feature = "use-tera")]

use failure::SyncFailure;
use http::header::HeaderValue;
use mime_guess::guess_mime_type_opt;
use serde::Serialize;
use std::borrow::Cow;
use tera::Tera;

use super::engine::{Engine, EngineImpl};
use renderer::Renderer;

pub trait AsTera {
    fn as_tera(&self) -> &Tera;
}

impl AsTera for Tera {
    fn as_tera(&self) -> &Tera {
        self
    }
}

impl<T: AsTera> AsTera for Box<T> {
    fn as_tera(&self) -> &Tera {
        (**self).as_tera()
    }
}

impl<T: AsTera> AsTera for ::std::rc::Rc<T> {
    fn as_tera(&self) -> &Tera {
        (**self).as_tera()
    }
}

impl<T: AsTera> AsTera for ::std::sync::Arc<T> {
    fn as_tera(&self) -> &Tera {
        (**self).as_tera()
    }
}

pub fn tera<T>(tera: T, name: impl Into<Cow<'static, str>>) -> Renderer<TeraEngine<T>>
where
    T: AsTera,
{
    Renderer::new(TeraEngine::new(tera, name))
}

#[derive(Debug)]
pub struct TeraEngine<T> {
    tera: T,
    name: Cow<'static, str>,
    content_type: Option<HeaderValue>,
}

impl<T> TeraEngine<T>
where
    T: AsTera,
{
    pub fn new(tera: T, name: impl Into<Cow<'static, str>>) -> TeraEngine<T> {
        let name = name.into();
        let content_type = guess_mime_type_opt(&*name).map(|mime| {
            mime.as_ref()
                .parse()
                .expect("should be a valid header value")
        });
        TeraEngine {
            tera,
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

impl<T, CtxT: Serialize> Engine<CtxT> for TeraEngine<T> where T: AsTera {}

impl<T, CtxT: Serialize> EngineImpl<CtxT> for TeraEngine<T>
where
    T: AsTera,
{
    type Body = String;
    type Error = SyncFailure<::tera::Error>;

    fn content_type_hint(&self, _: &CtxT) -> Option<HeaderValue> {
        self.content_type.clone()
    }

    fn render(&self, value: CtxT) -> Result<Self::Body, Self::Error> {
        self.tera
            .as_tera()
            .render(&self.name, &value)
            .map_err(SyncFailure::new)
    }
}

#[test]
fn test_tera() {
    use std::sync::Arc;

    #[derive(Debug, Serialize)]
    struct Context {
        name: String,
    }

    let mut tera = Tera::default();
    tera.add_raw_template("index.html", "{{ name }}").unwrap();

    let value = Context {
        name: "Alice".into(),
    };

    let engine = TeraEngine::new(Arc::new(tera), "index.html");
    let body = engine.render(value).unwrap();
    assert_eq!(body, "Alice");
}
