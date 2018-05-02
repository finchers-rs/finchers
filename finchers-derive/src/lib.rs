#![doc(html_root_url = "https://docs.rs/finchers-derive/0.11.0")]
#![recursion_limit = "128"]

extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
extern crate syn;

mod http_response;

use proc_macro::TokenStream;

#[proc_macro_derive(HttpResponse, attributes(status_code))]
pub fn derive_http_response(input: TokenStream) -> TokenStream {
    http_response::derive(input.into()).into()
}
