use failure;
use finchers::output::body::ResBody;
use http::header::HeaderValue;

/// A trait representing a template engine.
pub trait Engine<CtxT>: EngineImpl<CtxT> {}

pub trait EngineImpl<CtxT> {
    type Body: ResBody;
    type Error: Into<failure::Error>;

    #[allow(unused_variables)]
    fn content_type_hint(&self, ctx: &CtxT) -> Option<HeaderValue> {
        None
    }

    fn render(&self, ctx: CtxT) -> Result<Self::Body, Self::Error>;
}
