/// Represents the conversion from a path segment
pub trait FromParam: Sized {
    /// Try to convert a `str` to itself
    fn from_path(s: &str) -> Option<Self>;
}

macro_rules! impl_from_param {
    ($($t:ty),*) => {$(
        impl FromParam for $t {
            fn from_path(s: &str) -> Option<Self> {
                s.parse().ok()
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
