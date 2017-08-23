//! Definition of the trait `Endpoint` and its implementors

pub mod body;
pub mod combinator;
pub mod endpoint;
pub mod header;
pub mod method;
pub mod param;
pub mod path;

// re-exports
#[doc(inline)]
pub use self::endpoint::{Endpoint, EndpointError, EndpointResult};

#[doc(inline)]
pub use self::body::{body, json_body, FromBody};

#[doc(inline)]
pub use self::header::{header, header_opt};

#[doc(inline)]
pub use self::param::{param, param_opt};

#[doc(inline)]
pub use self::path::{path, path_seq, path_vec};

// TODO: more smart
#[doc(inline)]
pub use self::path::{isize_, string_, usize_, f32_, f64_, i16_, i32_, i64_, i8_, u32_, u64_, u8_};
