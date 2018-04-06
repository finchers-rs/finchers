#[macro_use]
extern crate finchers;
use finchers::endpoint::path::{FromSegment, Segment};

#[test]
fn unit_struct() {
    #[derive(Debug, FromSegment, PartialEq)]
    struct Id(u32);

    let s = Segment::from("42");
    assert_eq!(<Id as FromSegment>::from_segment(s).ok(), Some(Id(42)));
}

#[test]
fn generic_struct() {
    #[derive(Debug, FromSegment, PartialEq)]
    struct Id<T>(T);

    let s = Segment::from("42");
    assert_eq!(<Id<u32> as FromSegment>::from_segment(s).ok(), Some(Id(42)));
}

#[test]
fn generic_struct_2() {
    #[derive(Debug, FromSegment, PartialEq)]
    struct Id<T: Copy>(T);

    let s = Segment::from("42");
    assert_eq!(<Id<u32> as FromSegment>::from_segment(s).ok(), Some(Id(42)));
}

#[test]
fn generic_struct_3() {
    #[derive(Debug, FromSegment, PartialEq)]
    struct Id<T>(T)
    where
        T: Copy;

    let s = Segment::from("42");
    assert_eq!(<Id<u32> as FromSegment>::from_segment(s).ok(), Some(Id(42)));
}

use std::fmt;

#[test]
fn generic_struct_assoc() {
    trait Foo: 'static {
        type Item: ::std::fmt::Debug + PartialEq;
    }

    struct A;
    impl Foo for A {
        type Item = u32;
    }

    #[derive(FromSegment)]
    struct Id<T: Foo>(T::Item);

    impl<T: Foo> fmt::Debug for Id<T> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            fmt::Debug::fmt(&self.0, f)
        }
    }

    impl<T: Foo> PartialEq for Id<T> {
        fn eq(&self, other: &Self) -> bool {
            self.0.eq(&other.0)
        }
    }

    let s = Segment::from("42");
    assert_eq!(<Id<A> as FromSegment>::from_segment(s).ok(), Some(Id(42)));
}
