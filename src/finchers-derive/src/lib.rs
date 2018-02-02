extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate syn;

mod http_status;

use proc_macro::TokenStream;
use quote::ToTokens;

#[proc_macro_derive(HttpStatus, attributes(status_code))]
pub fn derive_http_status(input: TokenStream) -> TokenStream {
    let input: syn::DeriveInput = syn::parse(input).unwrap();
    http_status::Context::from(input).into_tokens().into()
}
