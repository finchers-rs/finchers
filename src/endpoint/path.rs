use std::borrow::Cow;
use std::iter::FromIterator;
use std::marker::PhantomData;
use std::str::FromStr;
use endpoint::{Endpoint, EndpointContext, IntoEndpoint};

#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct MatchPath<E> {
    kind: MatchPathKind,
    _marker: PhantomData<fn() -> E>,
}

#[derive(Debug, Clone, PartialEq)]
enum MatchPathKind {
    Segments(Vec<String>),
    AllSegments,
}
use self::MatchPathKind::*;

impl<E> Endpoint for MatchPath<E> {
    type Item = ();
    type Error = E;
    type Task = Result<Self::Item, Self::Error>;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Task> {
        match self.kind {
            Segments(ref segments) => {
                let mut matched = true;
                for segment in segments {
                    matched = matched && try_opt!(ctx.next_segment()) == segment;
                }
                if matched {
                    Some(Ok(()))
                } else {
                    None
                }
            }
            AllSegments => {
                let _ = ctx.take_segments();
                Some(Ok(()))
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ParseMatchError {
    EmptyString,
}

#[allow(missing_docs)]
pub fn match_<E>(s: &str) -> Result<MatchPath<E>, ParseMatchError> {
    let s = s.trim().trim_left_matches("/").trim_right_matches("/");
    let kind = if s == "*" {
        AllSegments
    } else {
        let mut segments = Vec::new();
        for segment in s.split("/").map(|s| s.trim()) {
            if segment.is_empty() {
                return Err(ParseMatchError::EmptyString);
            }
            segments.push(segment.into());
        }
        Segments(segments)
    };

    Ok(MatchPath {
        kind,
        _marker: PhantomData,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_single_segment() {
        assert_eq!(
            match_::<()>("foo").map(|m| m.kind),
            Ok(Segments(vec!["foo".to_owned()]))
        );
    }

    #[test]
    fn test_match_multi_segments() {
        assert_eq!(
            match_::<()>("foo/bar").map(|m| m.kind),
            Ok(Segments(vec!["foo".to_owned(), "bar".to_owned()]))
        );
    }

    #[test]
    fn test_match_all_segments() {
        assert_eq!(match_::<()>("*").map(|m| m.kind), Ok(AllSegments));
    }

    #[test]
    fn test_match_failure_empty() {
        assert_eq!(
            match_::<()>("").map(|m| m.kind),
            Err(ParseMatchError::EmptyString)
        );
    }

    #[test]
    fn test_match_failure_empty_2() {
        assert_eq!(
            match_::<()>("foo//bar").map(|m| m.kind),
            Err(ParseMatchError::EmptyString)
        );
    }
}

impl<'a, E> IntoEndpoint<(), E> for &'a str {
    type Endpoint = MatchPath<E>;
    fn into_endpoint(self) -> Self::Endpoint {
        match_(self).unwrap()
    }
}

impl<E> IntoEndpoint<(), E> for String {
    type Endpoint = MatchPath<E>;
    fn into_endpoint(self) -> Self::Endpoint {
        match_(&self).unwrap()
    }
}

impl<'a, E> IntoEndpoint<(), E> for Cow<'a, str> {
    type Endpoint = MatchPath<E>;
    fn into_endpoint(self) -> Self::Endpoint {
        match_(&*self).unwrap()
    }
}

#[allow(missing_docs)]
pub fn path<T: FromStr, E>() -> ExtractPath<T, E> {
    ExtractPath(PhantomData)
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct ExtractPath<T, E>(PhantomData<fn() -> (T, E)>);

impl<T, E> Endpoint for ExtractPath<T, E>
where
    T: FromStr,
    E: From<T::Err>,
{
    type Item = T;
    type Error = E;
    type Task = Result<Self::Item, Self::Error>;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Task> {
        match ctx.next_segment().map(|s| s.parse()) {
            Some(res) => Some(res.map_err(Into::into)),
            _ => return None,
        }
    }
}

#[allow(missing_docs)]
pub fn paths<I, T, E>() -> ExtractPaths<I, T, E>
where
    I: FromIterator<T>,
    T: FromStr,
{
    ExtractPaths(PhantomData)
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct ExtractPaths<I, T, E>(PhantomData<fn() -> (I, T, E)>);

impl<I, T, E> Endpoint for ExtractPaths<I, T, E>
where
    I: FromIterator<T>,
    T: FromStr,
    E: From<T::Err>,
{
    type Item = I;
    type Error = E;
    type Task = Result<Self::Item, Self::Error>;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Task> {
        match ctx.take_segments()
            .map(|s| s.map(|s| s.parse()).collect::<Result<_, _>>())
        {
            Some(res) => Some(res.map_err(Into::into)),
            _ => return None,
        }
    }
}
