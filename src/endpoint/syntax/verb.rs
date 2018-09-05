use std::ops::{BitOr, BitOrAssign};

use bitflags::bitflags;
use http::Method;

use super::Matched;
use crate::endpoint::{Context, Endpoint, EndpointError, EndpointResult};

/// Create an endpoint which checks if the verb of current request
/// is equal to the specified value.
pub fn verbs(allowed: Verbs) -> MatchVerbs {
    (MatchVerbs { allowed }).with_output::<()>()
}

#[allow(missing_docs)]
#[derive(Debug, Copy, Clone)]
pub struct MatchVerbs {
    allowed: Verbs,
}

impl<'a> Endpoint<'a> for MatchVerbs {
    type Output = ();
    type Future = Matched;

    fn apply(&'a self, ecx: &mut Context<'_>) -> EndpointResult<Self::Future> {
        if self.allowed.contains(ecx.input().method()) {
            Ok(Matched { _priv: () })
        } else {
            Err(EndpointError::method_not_allowed(self.allowed))
        }
    }
}

macro_rules! define_verbs {
    ($(
        ($name:ident, $METHOD:ident, $Endpoint:ident),
    )*) => {$(

        #[allow(missing_docs)]
        #[inline]
        pub fn $name() -> $Endpoint {
            $Endpoint {
                _priv: (),
            }
        }

        #[allow(missing_docs)]
        #[derive(Debug, Copy, Clone)]
        pub struct $Endpoint {
            _priv: (),
        }

        impl<'e> Endpoint<'e> for $Endpoint {
            type Output = ();
            type Future = Matched;

            #[inline]
            fn apply(&'e self, ecx: &mut Context<'_>) -> EndpointResult<Self::Future> {
                if *ecx.input().method() == Method::$METHOD {
                    Ok(Matched { _priv: () })
                } else {
                    Err(EndpointError::method_not_allowed(Verbs::$METHOD))
                }
            }
        }
    )*};
}

define_verbs! {
    (get, GET, MatchVerbGet),
    (post, POST, MatchVerbPost),
    (put, PUT, MatchVerbPut),
    (delete, DELETE, MatchVerbDelete),
    (head, HEAD, MatchVerbHead),
    (options, OPTIONS, MatchVerbOptions),
    (connect, CONNECT, MatchVerbConnect),
    (patch, PATCH, MatchVerbPatch),
    (trace, TRACE, MatchVerbTrace),
}

/// A collection type which represents a set of allowed HTTP methods.
#[derive(Debug, Clone, Copy)]
pub struct Verbs(Methods);

bitflags! {
    struct Methods: u32 {
        const GET         = 0b_0000_0000_0001;
        const POST        = 0b_0000_0000_0010;
        const PUT         = 0b_0000_0000_0100;
        const DELETE      = 0b_0000_0000_1000;
        const HEAD        = 0b_0000_0001_0000;
        const OPTIONS     = 0b_0000_0010_0000;
        const CONNECT     = 0b_0000_0100_0000;
        const PATCH       = 0b_0000_1000_0000;
        const TRACE       = 0b_0001_0000_0000;
    }
}

macro_rules! define_allowed_methods_constructors {
    ($($METHOD:ident,)*) => {$(
        #[allow(missing_docs)]
        pub const $METHOD: Verbs = Verbs(Methods::$METHOD);
    )*};
}

impl Verbs {
    define_allowed_methods_constructors![
        GET, POST, PUT, DELETE, HEAD, OPTIONS, CONNECT, PATCH, TRACE,
    ];

    #[allow(missing_docs)]
    pub fn single(method: &Method) -> Option<Verbs> {
        macro_rules! pat {
            ($($METHOD:ident),*) => {
                match method {
                    $(
                        ref m if *m == Method::$METHOD => Some(Verbs::$METHOD),
                    )*
                    _ => None,
                }
            }
        }
        pat!(GET, POST, PUT, DELETE, HEAD, OPTIONS, CONNECT, PATCH, TRACE)
    }

    #[allow(missing_docs)]
    #[inline]
    pub fn any() -> Verbs {
        Verbs(Methods::all())
    }

    pub(crate) fn contains(&self, method: &Method) -> bool {
        macro_rules! compare_methods {
            ($($METHOD:ident),*) => {
                match method {
                    $(
                        m if *m == Method::$METHOD => self.0.contains(Methods::$METHOD),
                    )*
                    _ => false,
                }
            }
        }
        compare_methods![GET, POST, PUT, DELETE, HEAD, OPTIONS, CONNECT, PATCH, TRACE]
    }
}

impl BitOr for Verbs {
    type Output = Verbs;

    #[inline]
    fn bitor(self, other: Verbs) -> Self::Output {
        Verbs(self.0 | other.0)
    }
}

impl BitOrAssign for Verbs {
    #[inline]
    fn bitor_assign(&mut self, other: Verbs) {
        self.0 |= other.0;
    }
}

impl IntoIterator for Verbs {
    type Item = &'static Method;
    type IntoIter = VerbsIter;

    fn into_iter(self) -> Self::IntoIter {
        VerbsIter {
            allowed: self.0,
            cursor: Methods::GET,
        }
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct VerbsIter {
    allowed: Methods,
    cursor: Methods,
}

impl Iterator for VerbsIter {
    type Item = &'static Method;

    fn next(&mut self) -> Option<Self::Item> {
        macro_rules! dump_method {
            ($m:expr => [$($METHOD:ident),*]) => {$(
                if $m.contains(Methods::$METHOD) { return Some(&Method::$METHOD) }
            )*}
        }
        loop {
            let masked = self.allowed & self.cursor;
            self.cursor = Methods::from_bits_truncate(self.cursor.bits() << 1);
            if self.cursor.is_empty() {
                return None;
            }
            dump_method!(masked => [
                GET,
                POST,
                PUT,
                DELETE,
                HEAD,
                OPTIONS,
                CONNECT,
                PATCH,
                TRACE
            ]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_methods_single_get() {
        let methods: Vec<Method> = Verbs::GET.into_iter().cloned().collect();
        assert_eq!(methods, vec![Method::GET]);
    }

    #[test]
    fn test_methods_two_methods() {
        let methods: Vec<Method> = (Verbs::GET | Verbs::POST).into_iter().cloned().collect();
        assert_eq!(methods, vec![Method::GET, Method::POST]);
    }
}
