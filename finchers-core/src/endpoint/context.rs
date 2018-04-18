use Input;
use std::ops::Deref;

/// A context during the routing.
#[derive(Debug, Clone)]
pub struct Context<'a> {
    input: &'a Input,
    segments: Segments<'a>,
}

impl<'a> Context<'a> {
    pub(crate) fn new(input: &'a Input) -> Self {
        Context {
            input: input,
            segments: Segments::from(input.request().uri().path()),
        }
    }

    pub fn input(&self) -> &'a Input {
        self.input
    }

    /// Returns the reference of remaining path segments
    pub fn segments(&mut self) -> &mut Segments<'a> {
        &mut self.segments
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_segments_from_root_path() {
        let mut segments = Segments::from("/");
        assert_eq!(segments.remaining_path(), "");
        assert_eq!(segments.next().map(|s| s.as_str()), None);
    }

}
