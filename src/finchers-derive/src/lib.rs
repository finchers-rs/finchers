extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;

use syn::{DeriveInput, Ident, Lit, Meta};
use quote::{ToTokens, Tokens};

#[derive(Debug)]
struct Attr {
    status_code: Ident,
}

impl Attr {
    fn from_attributes(attrs: &[syn::Attribute]) -> Self {
        let mut status_code = None;
        for meta in attrs.iter().filter_map(|a| a.interpret_meta()) {
            match meta {
                Meta::NameValue(nv) => {
                    if nv.ident != "status_code" {
                        panic!("supported only 'status_code = \"STATUS_CODE\"'");
                    }
                    if let Lit::Str(s) = nv.lit {
                        status_code = Some(s.value().into());
                    } else {
                        panic!("RHS must be a string literal");
                    }
                }
                _ => continue,
            }
        }
        Attr {
            status_code: status_code.unwrap_or("OK".into()),
        }
    }
}

#[derive(Debug)]
struct Body {
    ident: Ident,
    kind: BodyKind,
}

#[derive(Debug)]
enum BodyKind {
    Struct(Attr),
    Enum(Vec<(Ident, Attr)>),
}

impl Body {
    pub fn from_derive_input(input: &DeriveInput) -> Self {
        match input.data {
            syn::Data::Struct(..) => {
                let attr = Attr::from_attributes(&input.attrs);
                Body {
                    ident: input.ident.clone(),
                    kind: BodyKind::Struct(attr),
                }
            }
            syn::Data::Enum(ref data) => {
                let mut variants = vec![];
                for variant in &data.variants {
                    let attr = Attr::from_attributes(&variant.attrs);
                    variants.push((variant.ident.clone(), attr));
                }
                Body {
                    ident: input.ident.clone(),
                    kind: BodyKind::Enum(variants),
                }
            }
            syn::Data::Union(..) => panic!("union does not supported"),
        }
    }
}

impl ToTokens for Body {
    fn to_tokens(&self, tokens: &mut Tokens) {
        match self.kind {
            BodyKind::Struct(ref attr) => {
                let status_code = &attr.status_code;
                tokens.append_all(quote!(StatusCode::#status_code));
            }
            BodyKind::Enum(ref variants) => {
                let ident = &self.ident;
                let inner = variants.into_iter().map(|&(variant, ref attr)| {
                    let status_code = &attr.status_code;
                    quote!(#ident :: #variant => StatusCode::#status_code)
                });
                tokens.append_all(quote!(
                    match *self {
                        #(#inner,)*
                    }
                ));
            }
        }
    }
}

#[proc_macro_derive(HttpResponse, attributes(status_code))]
pub fn derive_http_response(input: TokenStream) -> TokenStream {
    let input: DeriveInput = syn::parse(input).unwrap();
    let body = Body::from_derive_input(&input);
    let ident = &input.ident;
    let dummy_mod = Ident::from(format!("__impl_http_response_for_{}", ident));
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    TokenStream::from(quote! {
        #[allow(non_snake_case)]
        mod #dummy_mod {
            use finchers::core::HttpResponse;
            use finchers::http::StatusCode;

            impl #impl_generics HttpResponse for #ident #ty_generics #where_clause {
                fn status_code(&self) -> StatusCode {
                    #body
                }
            }
        }
    })
}
