use request::RequestInfo;

#[derive(Debug)]
pub struct ResponderContext<'a> {
    pub request: &'a RequestInfo,
}
