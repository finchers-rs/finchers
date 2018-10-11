#![deny(warnings)]
#![doc(test(attr(deny(warnings))))]

#[macro_use]
extern crate doubter;

doubter! {
    file = "../../README.md",
}
