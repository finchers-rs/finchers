use std::fmt;
use syn::{self, DeriveInput, Generics, Ident, Lit, Meta};
use quote::{ToTokens, Tokens};

#[derive(Debug)]
pub struct Attr {
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
pub enum Body {
    Struct(Attr),
    Enum(Vec<(Ident, Attr)>),
}

impl Body {
    pub fn from_data(data: syn::Data, attrs: Vec<syn::Attribute>) -> Self {
        match data {
            syn::Data::Struct(..) => {
                let attr = Attr::from_attributes(&attrs);
                Body::Struct(attr)
            }
            syn::Data::Enum(data) => {
                let mut variants = vec![];
                for variant in data.variants {
                    let attr = Attr::from_attributes(&variant.attrs);
                    variants.push((variant.ident, attr));
                }
                Body::Enum(variants)
            }
            syn::Data::Union(..) => panic!("union does not supported"),
        }
    }

    pub fn to_tokens(&self, ident: &Ident) -> Tokens {
        match *self {
            Body::Struct(ref attr) => {
                let status_code = &attr.status_code;
                quote!(StatusCode::#status_code)
            }
            Body::Enum(ref variants) => {
                let inner = variants.into_iter().map(|&(variant, ref attr)| {
                    let status_code = &attr.status_code;
                    quote!(#ident :: #variant => StatusCode::#status_code)
                });
                quote!(
                    match *self {
                        #(#inner,)*
                    }
                )
            }
        }
    }
}

pub struct Context {
    ident: Ident,
    generics: Generics,
    body: Body,
}

impl fmt::Debug for Context {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Context")
            .field("ident", &self.ident)
            .field("generics", &"[generics]")
            .field("body", &self.body)
            .finish()
    }
}

impl From<DeriveInput> for Context {
    fn from(input: DeriveInput) -> Self {
        Context {
            ident: input.ident,
            generics: input.generics,
            body: Body::from_data(input.data, input.attrs),
        }
    }
}

impl Context {
    fn dummy_module_ident(&self) -> Ident {
        Ident::from(format!("__impl_http_status_for_{}", self.ident))
    }
}

impl ToTokens for Context {
    fn to_tokens(&self, tokens: &mut Tokens) {
        let ident = &self.ident;
        let dummy_mod = self.dummy_module_ident();
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();
        let body = self.body.to_tokens(&self.ident);
        tokens.append_all(quote! {
            #[allow(non_snake_case)]
            mod #dummy_mod {
                use finchers::core::HttpStatus;
                use finchers::http::StatusCode;

                impl #impl_generics HttpStatus for #ident #ty_generics #where_clause {
                    fn status_code(&self) -> StatusCode {
                        #body
                    }
                }
            }
        });
    }
}
