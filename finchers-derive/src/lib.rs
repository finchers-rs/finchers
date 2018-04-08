#![recursion_limit = "128"]

extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
extern crate syn;

mod http_status;

use proc_macro::TokenStream;

#[proc_macro_derive(HttpStatus, attributes(status_code))]
pub fn derive_http_status(input: TokenStream) -> TokenStream {
    http_status::derive(input.into()).into()
}
