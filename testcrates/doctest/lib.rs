#![deny(warnings)]
#![doc(test(attr(deny(warnings))))]

#[macro_use]
extern crate doubter;

generate_doc_tests! {
    include = "../../README.md",
}
