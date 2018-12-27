#![cfg(feature = "use-askama")]

use super::engine::{Engine, EngineImpl};
use renderer::Renderer;

use askama::Template;
use http::header::HeaderValue;
use mime_guess::get_mime_type_str;
use std::marker::PhantomData;

pub fn askama<CtxT: Template>() -> Renderer<AskamaEngine<CtxT>> {
    Renderer::new(AskamaEngine::default())
}

#[derive(Debug)]
pub struct AskamaEngine<CtxT> {
    content_type_cache: Option<HeaderValue>,
    _marker: PhantomData<fn(CtxT)>,
}

impl<CtxT> Default for AskamaEngine<CtxT> {
    fn default() -> Self {
        AskamaEngine {
            content_type_cache: None,
            _marker: PhantomData,
        }
    }
}

impl<CtxT: Template> AskamaEngine<CtxT> {
    /// Precompute the value of content-type by using the given instance of context.
    pub fn precompute_content_type(&mut self, hint: &CtxT) {
        self.content_type_cache = hint.extension().and_then(|ext| {
            get_mime_type_str(ext)
                .map(|mime_str| mime_str.parse().expect("should be a valid header value"))
        });
    }
}

impl<CtxT: Template> Engine<CtxT> for AskamaEngine<CtxT> {}

impl<CtxT: Template> EngineImpl<CtxT> for AskamaEngine<CtxT> {
    type Body = String;
    type Error = ::askama::Error;

    // FIXME: cache parsed value
    fn content_type_hint(&self, value: &CtxT) -> Option<HeaderValue> {
        self.content_type_cache.clone().or_else(|| {
            let ext = value.extension()?;
            get_mime_type_str(ext)?.parse().ok()
        })
    }

    fn render(&self, value: CtxT) -> Result<Self::Body, Self::Error> {
        value.render()
    }
}

#[test]
fn test_askama() {
    use askama::Error;
    use std::fmt;

    #[derive(Debug)]
    struct Context {
        name: String,
    }

    impl Template for Context {
        fn render_into(&self, writer: &mut dyn fmt::Write) -> Result<(), Error> {
            write!(writer, "{}", self.name).map_err(Into::into)
        }

        fn extension(&self) -> Option<&str> {
            Some("html")
        }
    }

    let value = Context {
        name: "Alice".into(),
    };

    let engine = AskamaEngine::default();
    assert_matches!(
        engine.content_type_hint(&value),
        Some(ref h) if h == "text/html"
    );
    assert_matches!(
        engine.render(value),
        Ok(ref body) if body == "Alice"
    );
}
