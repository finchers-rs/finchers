use std::error::Error;
use egg_mode;

#[derive(Debug)]
pub enum SearchTwitterError {
    Endpoint(Box<Error>),
    Twitter(egg_mode::error::Error),
}

impl<E: Error + 'static> From<E> for SearchTwitterError {
    fn from(error: E) -> Self {
        SearchTwitterError::Endpoint(Box::new(error))
    }
}
