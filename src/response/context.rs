use request::Request;

#[derive(Debug)]
pub struct ResponderContext<'a> {
    pub request: &'a Request,
}
