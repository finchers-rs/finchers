#![cfg_attr(feature = "unstable", feature(conservative_impl_trait))]
#![allow(unused_variables)]

#[cfg_attr(feature = "unstable", macro_use)]
extern crate finchers;
extern crate finchers_json;
#[cfg(feature = "unstable")]
#[macro_use]
extern crate serde_derive;

#[cfg(feature = "unstable")]
mod imp;

#[cfg(not(feature = "unstable"))]
mod imp {
    pub fn main() {
        println!("This example works only if the feature 'unstable' is enable.");
    }
}

fn main() {
    imp::main()
}
