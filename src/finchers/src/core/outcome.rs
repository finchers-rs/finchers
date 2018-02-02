#![allow(missing_docs)]

use super::Error;

#[derive(Debug)]
pub enum Outcome<T> {
    Ok(T),
    Err(Error),
    NoRoute,
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
}
