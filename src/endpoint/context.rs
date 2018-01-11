use std::ops::Deref;
use http::{Cookies, Request};

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

/// The type of returned value of `Segments::next()`
#[derive(Debug, Copy, Clone)]
pub struct Segment<'a> {
    s: &'a str,
    start: usize,
    end: usize,
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

#[cfg(test)]
mod tests {
    use super::Segments;

    #[test]
    fn test_segments() {
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
    fn test_root() {
        let mut segments = Segments::from("/");
        assert_eq!(segments.remaining_path(), "");
        assert_eq!(segments.next().map(|s| s.as_str()), None);
    }
}

/// A context during the routing.
#[derive(Debug, Clone)]
pub struct EndpointContext<'a> {
    request: &'a Request,
    cookies: &'a Cookies,
    segments: Segments<'a>,
}

impl<'a> EndpointContext<'a> {
    pub(crate) fn new(request: &'a Request, cookies: &'a Cookies) -> Self {
        EndpointContext {
            request,
            cookies,
            segments: Segments::from(request.path()),
        }
    }

    /// Returns the reference of HTTP request
    pub fn request(&self) -> &Request {
        self.request
    }

    /// Returns the reference of Cookies
    pub fn cookies(&self) -> &Cookies {
        self.cookies
    }

    /// Returns the reference of remaining path segments
    pub fn segments(&mut self) -> &mut Segments<'a> {
        &mut self.segments
    }
}
