#![feature(rust_2018_preview)]
#![feature(pin, arbitrary_self_types, futures_api)]

extern crate bytes;
extern crate failure;
extern crate finchers;
extern crate http;

macro_rules! assert_matches {
    ($e:expr, $($t:tt)+) => {
        assert_matches!(@hack match $e {
            $($t)+ => {},
            ref e => panic!("assertion failed: `{:?}` does not match `{}`", e, stringify!($($t)+)),
        })
    };
    (@hack $v:expr) =>  { $v };
}

//mod codegen;
mod endpoint;
mod endpoints;
