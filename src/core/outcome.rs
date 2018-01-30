#![allow(missing_docs)]

use super::Error;

#[derive(Debug)]
pub enum Outcome<T> {
    Ok(T),
    Err(Error),
    NoRoute,
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
