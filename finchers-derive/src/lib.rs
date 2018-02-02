extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
extern crate syn;

mod http_status;

use proc_macro::TokenStream;
use quote::ToTokens;

#[proc_macro_derive(HttpStatus, attributes(status_code))]
pub fn derive_http_status(input: TokenStream) -> TokenStream {
    let input: syn::DeriveInput = syn::parse(input).unwrap();
    let context = http_status::Context::from(input);
    context.into_tokens().into()
}
