use std::error::Error;
use std::ops::Deref;
use std::path::PathBuf;
use std::str::FromStr;
use never::Never;

/// An iterator of remaning path segments.
#[derive(Debug, Copy, Clone)]
pub struct Segments<'a> {
    path: &'a str,
    pos: usize,
    popped: usize,
}

impl<'a> From<&'a str> for Segments<'a> {
    fn from(path: &'a str) -> Self {
        debug_assert!(!path.is_empty());
        debug_assert_eq!(path.chars().next(), Some('/'));
        Segments {
            path,
            pos: 1,
            popped: 0,
        }
    }
}

impl<'a> Segments<'a> {
    /// Returns the remaining path in this segments
    #[inline]
    pub fn remaining_path(&self) -> &'a str {
        &self.path[self.pos..]
    }

    /// Returns the cursor position in the original path
    #[inline]
    pub fn position(&self) -> usize {
        self.pos
    }

    /// Returns the number of segments already popped
    #[inline]
    pub fn popped(&self) -> usize {
        self.popped
    }
}

impl<'a> Iterator for Segments<'a> {
    type Item = Segment<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos == self.path.len() {
            return None;
        }
        if let Some(offset) = self.path[self.pos..].find('/') {
            let segment = Segment {
                s: &self.path[self.pos..self.pos + offset],
                start: self.pos,
                end: self.pos + offset,
            };
            self.pos += offset + 1;
            self.popped += 1;
            Some(segment)
        } else {
            let segment = Segment {
                s: &self.path[self.pos..],
                start: self.pos,
                end: self.path.len(),
            };
            self.pos = self.path.len();
            self.popped += 1;
            Some(segment)
        }
    }
}

/// A path segment in HTTP requests
#[derive(Debug, Copy, Clone)]
pub struct Segment<'a> {
    s: &'a str,
    start: usize,
    end: usize,
}

impl<'a> From<&'a str> for Segment<'a> {
    fn from(s: &'a str) -> Self {
        Segment {
            s,
            start: 0,
            end: s.len(),
        }
    }
}

impl<'a> Segment<'a> {
    /// Yields the underlying `str` slice.
    #[inline]
    pub fn as_str(&self) -> &'a str {
        self.s
    }

    /// Returns the start position of this segment in the original path
    #[inline]
    pub fn start(&self) -> usize {
        self.start
    }

    /// Returns the end position of this segment in the original path
    #[inline]
    pub fn end(&self) -> usize {
        self.end
    }
}

impl<'a> AsRef<[u8]> for Segment<'a> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_str().as_bytes()
    }
}

impl<'a> AsRef<str> for Segment<'a> {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<'a> Deref for Segment<'a> {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        self.as_str()
    }
}

/// Represents the conversion from `Segment`
pub trait FromSegment: 'static + Sized {
    /// The error type returned from `from_segment`
    type Err: Error + Send + 'static;

    /// Create the instance of `Self` from a path segment
    fn from_segment(segment: Segment) -> Result<Self, Self::Err>;
}

macro_rules! impl_from_segment_from_str {
    ($($t:ty,)*) => {$(
        impl FromSegment for $t {
            type Err = <$t as FromStr>::Err;

            #[inline]
            fn from_segment(segment: Segment) -> Result<Self, Self::Err> {
                FromStr::from_str(&*segment)
            }
        }
    )*};
}

impl_from_segment_from_str! {
    String, bool, f32, f64,
    i8, i16, i32, i64, isize,
    u8, u16, u32, u64, usize,
    ::std::net::IpAddr,
    ::std::net::Ipv4Addr,
    ::std::net::Ipv6Addr,
    ::std::net::SocketAddr,
    ::std::net::SocketAddrV4,
    ::std::net::SocketAddrV6,
}

impl<T: FromSegment> FromSegment for Option<T> {
    type Err = Never;

    #[inline]
    fn from_segment(segment: Segment) -> Result<Self, Self::Err> {
        Ok(FromSegment::from_segment(segment).ok())
    }
}

impl<T: FromSegment> FromSegment for Result<T, T::Err> {
    type Err = Never;

    #[inline]
    fn from_segment(segment: Segment) -> Result<Self, Self::Err> {
        Ok(FromSegment::from_segment(segment))
    }
}

/// Represents the conversion from `Segments`
pub trait FromSegments: 'static + Sized {
    /// The error type from `from_segments`
    type Err: Error + Send + 'static;

    /// Create the instance of `Self` from the remaining path segments
    fn from_segments(segments: &mut Segments) -> Result<Self, Self::Err>;
}

impl<T: FromSegment> FromSegments for Vec<T> {
    type Err = T::Err;

    fn from_segments(segments: &mut Segments) -> Result<Self, Self::Err> {
        segments.into_iter().map(|s| T::from_segment(s)).collect()
    }
}

impl FromSegments for String {
    type Err = Never;

    fn from_segments(segments: &mut Segments) -> Result<Self, Self::Err> {
        let s = segments.remaining_path().to_owned();
        let _ = segments.last();
        Ok(s)
    }
}

impl FromSegments for PathBuf {
    type Err = Never;

    fn from_segments(segments: &mut Segments) -> Result<Self, Self::Err> {
        let s = PathBuf::from(segments.remaining_path());
        let _ = segments.last();
        Ok(s)
    }
}

impl<T: FromSegments> FromSegments for Option<T> {
    type Err = Never;

    fn from_segments(segments: &mut Segments) -> Result<Self, Self::Err> {
        Ok(FromSegments::from_segments(segments).ok())
    }
}

impl<T: FromSegments> FromSegments for Result<T, T::Err> {
    type Err = Never;

    fn from_segments(segments: &mut Segments) -> Result<Self, Self::Err> {
        Ok(FromSegments::from_segments(segments))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segments() {
        let mut segments = Segments::from("/foo/bar.txt");
        assert_eq!(segments.remaining_path(), "foo/bar.txt");
        assert_eq!(segments.next().map(|s| s.as_str()), Some("foo"));
        assert_eq!(segments.remaining_path(), "bar.txt");
        assert_eq!(segments.next().map(|s| s.as_str()), Some("bar.txt"));
        assert_eq!(segments.remaining_path(), "");
        assert_eq!(segments.next().map(|s| s.as_str()), None);
        assert_eq!(segments.remaining_path(), "");
        assert_eq!(segments.next().map(|s| s.as_str()), None);
    }

    #[test]
    fn segments_from_root_path() {
        let mut segments = Segments::from("/");
        assert_eq!(segments.remaining_path(), "");
        assert_eq!(segments.next().map(|s| s.as_str()), None);
    }

    #[test]
    fn from_segments() {
        let mut segments = Segments::from("/foo/bar.txt");
        let result = FromSegments::from_segments(&mut segments);
        assert_eq!(result, Ok(PathBuf::from("foo/bar.txt")));
    }
}
