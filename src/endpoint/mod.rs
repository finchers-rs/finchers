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
pub use self::endpoint::Endpoint;

#[doc(inline)]
pub use self::body::{body, FromBody};

#[doc(inline)]
pub use self::header::header;

#[doc(inline)]
pub use self::param::param;

#[doc(inline)]
pub use self::path::{path, path_end, path_vec};

// TODO: more smart
#[doc(inline)]
pub use self::path::{end_, string_, string_vec_, f32_, f32_vec_, f64_, f64_vec_, i32_, i32_vec_, i64_, i64_vec_, u32_,
                     u32_vec_, u64_, u64_vec_};
