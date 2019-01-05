//! Components for parsing the incoming HTTP request.

mod encoded;
mod header;

pub use self::encoded::{EncodedStr, FromEncodedStr};
pub use self::header::FromHeaderValue;
