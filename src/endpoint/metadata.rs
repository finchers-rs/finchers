#![allow(missing_docs)]

use super::Endpoint;

pub trait EndpointMetadata {
    type Output: OutputMetadata;

    fn dump(&self, meta: Metadata<'m>) -> MetadataOut<'m> {
        meta.finish::<Self::Output>()
    }
}

pub struct Out {
    _priv: (),
}

struct Inner {
    routes: Vec<Route>,
}

#[derive(Default)]
struct Route {
    segments: Vec<Segment>,
}

enum Segment {
    Static(String),
    Param(String, String),
}

pub struct Metadata<'m> {
    inner: &'m mut Inner,
    route: Route,
}

impl Metadata<'m> {
    pub fn route<'b>(&'b mut self) -> Metadata<'b> {
        Metadata {
            inner: &mut self.inner,
            route: Route::default(),
        }
    }

    pub fn path(&mut self, s: &str) {
        self.route.segments.push(Segment::Static(s.into()));
    }

    pub fn param(&mut self, name: &str, ty: &str) {
        self.route
            .segments
            .push(Segment::Param(name.into(), ty.into()));
    }

    pub fn finish<T: OutputMetadata>(self) -> MetadataOut<'m> {
        let Metadata { inner, route } = self;
        inner.routes.push(route);
        MetadataOut { inner }
    }
}

pub trait OutputMetadata {
    fn dump(meta: MetadataOut);
}

pub struct MetadataOut<'m> {
    inner: &'m mut Inner,
}
