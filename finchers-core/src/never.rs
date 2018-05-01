use error::HttpError;
use http::{Response, StatusCode};
use input::Input;
use output::ResponseBody;
use std::{error, fmt};

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Ord, Eq)]
pub enum Never {}

impl Never {
    pub fn never_into<T>(self) -> T {
        match self {}
    }
}

impl fmt::Display for Never {
    fn fmt(&self, _: &mut fmt::Formatter) -> fmt::Result {
        match *self {}
    }
}

impl error::Error for Never {
    fn description(&self) -> &str {
        match *self {}
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {}
    }
}

impl HttpError for Never {
    fn status_code(&self) -> StatusCode {
        match *self {}
    }

    fn to_response(&self, _: &Input) -> Option<Response<ResponseBody>> {
        match *self {}
    }
}
