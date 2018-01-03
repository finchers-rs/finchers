use http::{Cookies, Request};

/// An iterator of remaning path segments.
#[derive(Debug, Clone)]
pub struct Segments<'a> {
    path: &'a str,
    pos: usize,
}

impl<'a> From<&'a str> for Segments<'a> {
    fn from(path: &'a str) -> Self {
        debug_assert!(!path.is_empty());
        debug_assert_eq!(path.chars().next(), Some('/'));
        Segments { path, pos: 1 }
    }
}

impl<'a> Segments<'a> {
    /// Returns the remaining path in this segments
    pub fn as_str(&self) -> &'a str {
        &self.path[self.pos..]
    }
}

impl<'a> Iterator for Segments<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos == self.path.len() {
            return None;
        }
        if let Some(offset) = self.path[self.pos..].find('/') {
            let segment = &self.path[self.pos..self.pos + offset];
            self.pos += offset + 1;
            Some(segment)
        } else {
            let segment = &self.path[self.pos..];
            self.pos = self.path.len();
            Some(segment)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Segments;

    #[test]
    fn test_segments() {
        let mut segments = Segments::from("/foo/bar.txt");
        assert_eq!(segments.as_str(), "foo/bar.txt");
        assert_eq!(segments.next(), Some("foo"));
        assert_eq!(segments.as_str(), "bar.txt");
        assert_eq!(segments.next(), Some("bar.txt"));
        assert_eq!(segments.as_str(), "");
        assert_eq!(segments.next(), None);
        assert_eq!(segments.as_str(), "");
        assert_eq!(segments.next(), None);
    }

    #[test]
    fn test_root() {
        let mut segments = Segments::from("/");
        assert_eq!(segments.as_str(), "");
        assert_eq!(segments.next(), None);
    }
}

/// A context during the routing.
#[derive(Debug, Clone)]
pub struct EndpointContext<'a> {
    request: &'a Request,
    cookies: &'a Cookies,
    segments: Option<Segments<'a>>,
}

impl<'a> EndpointContext<'a> {
    pub(crate) fn new(request: &'a Request, cookies: &'a Cookies) -> Self {
        EndpointContext {
            request,
            cookies,
            segments: Some(Segments::from(request.path())),
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

    /// Pop and return the front element of path segments.
    pub fn next_segment(&mut self) -> Option<&str> {
        self.segments.as_mut().and_then(|r| r.next())
    }

    /// Collect and return the remaining path segments, if available
    pub fn take_segments(&mut self) -> Option<Segments<'a>> {
        self.segments.take()
    }
}
