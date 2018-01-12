#![cfg_attr(feature = "unstable", feature(conservative_impl_trait))]

#[cfg(feature = "unstable")]
#[macro_use]
extern crate error_chain;
#[cfg(feature = "unstable")]
#[macro_use]
extern crate finchers;
#[cfg(feature = "unstable")]
#[macro_use]
extern crate lazy_static;
#[cfg(feature = "unstable")]
#[macro_use]
extern crate serde_derive;
#[cfg(feature = "unstable")]
extern crate serde_json;

#[cfg(feature = "unstable")]
mod imp;

#[cfg(not(feature = "unstable"))]
mod imp {
    pub fn main() {
        println!("This example is available only on nightly Rust compiler with 'unstable' option.");
    }
}

fn main() {
    imp::main()
}
