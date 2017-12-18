use std::str::FromStr;

/// Represents the conversion from a path segment
pub trait FromParam: Sized {
    /// The error type of `from_param()`
    type Error;

    /// Try to convert a `str` to itself
    fn from_param(s: &str) -> Result<Self, Self::Error>;
}

macro_rules! impl_from_param {
    ($($t:ty),*) => {$(
        impl FromParam for $t {
            type Error = <$t as FromStr>::Err;

            fn from_param(s: &str) -> Result<Self, Self::Error> {
                s.parse()
            }
        }
    )*}
}

impl_from_param!(
    i8,
    u8,
    i16,
    u16,
    i32,
    u32,
    i64,
    u64,
    isize,
    usize,
    f32,
    f64,
    String
);
