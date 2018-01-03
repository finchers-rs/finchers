use std::fmt;
use std::error::Error;

#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct EmptyHeader(pub(crate) &'static str);

impl fmt::Display for EmptyHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "The header '{}' is not given", self.0)
    }
}

impl Error for EmptyHeader {
    fn description(&self) -> &str {
        "empty header"
    }
}
