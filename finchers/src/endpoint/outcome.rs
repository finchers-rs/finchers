#![allow(missing_docs)]

use errors::Error;

#[derive(Debug)]
pub enum Outcome<T> {
    Ok(T),
    Err(Error),
    NoRoute,
}

impl<T> From<T> for Outcome<T> {
    fn from(input: T) -> Outcome<T> {
        Outcome::Ok(input)
    }
}

impl<T> From<Option<T>> for Outcome<T> {
    fn from(input: Option<T>) -> Outcome<T> {
        match input {
            Some(item) => Outcome::Ok(item),
            None => Outcome::NoRoute,
        }
    }
}

impl<T, E: Into<Error>> From<Result<T, E>> for Outcome<T> {
    fn from(input: Result<T, E>) -> Outcome<T> {
        match input {
            Ok(item) => Outcome::Ok(item),
            Err(err) => Outcome::Err(err.into()),
        }
    }
}

impl<T, E: Into<Error>> From<Result<Option<T>, E>> for Outcome<T> {
    fn from(input: Result<Option<T>, E>) -> Outcome<T> {
        match input {
            Ok(Some(item)) => Outcome::Ok(item),
            Ok(None) => Outcome::NoRoute,
            Err(err) => Outcome::Err(err.into()),
        }
    }
}

impl<T, E: Into<Error>> From<Option<Result<T, E>>> for Outcome<T> {
    fn from(input: Option<Result<T, E>>) -> Outcome<T> {
        match input {
            Some(Ok(item)) => Outcome::Ok(item),
            Some(Err(err)) => Outcome::Err(err.into()),
            None => Outcome::NoRoute,
        }
    }
}

impl<T> Outcome<T> {
    #[inline]
    pub fn is_ok(&self) -> bool {
        match *self {
            Outcome::Ok(..) => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_err(&self) -> bool {
        match *self {
            Outcome::Err(..) => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_noroute(&self) -> bool {
        match *self {
            Outcome::NoRoute => true,
            _ => false,
        }
    }

    #[inline]
    pub fn ok(self) -> Option<T> {
        match self {
            Outcome::Ok(item) => Some(item),
            _ => None,
        }
    }

    #[inline]
    pub fn err(self) -> Option<Error> {
        match self {
            Outcome::Err(err) => Some(err),
            _ => None,
        }
    }

    #[inline]
    pub fn into_result(self) -> Result<Option<T>, Error> {
        match self {
            Outcome::Ok(item) => Ok(Some(item)),
            Outcome::Err(err) => Err(err),
            Outcome::NoRoute => Ok(None),
        }
    }
}

impl<T> Into<Result<Option<T>, Error>> for Outcome<T> {
    fn into(self) -> Result<Option<T>, Error> {
        self.into_result()
    }
}
