use proc_macro2::{Span, Term, TokenStream, TokenTree};
use quote::{ToTokens, Tokens};
use std::fmt;
use syn::{self, DeriveInput, Generics, Ident, Meta};

const SUPPORTED_STATUSES: &[(u16, &str)] = &[
    (100, "CONTINUE"),
    (101, "SWITCHING_PROTOCOLS"),
    (102, "PROCESSING"),
    (200, "OK"),
    (201, "CREATED"),
    (202, "ACCEPTED"),
    (203, "NON_AUTHORITATIVE_INFORMATION"),
    (204, "NO_CONTENT"),
    (205, "RESET_CONTENT"),
    (206, "PARTIAL_CONTENT"),
    (207, "MULTI_STATUS"),
    (208, "ALREADY_REPORTED"),
    (226, "IM_USED"),
    (300, "MULTIPLE_CHOICES"),
    (301, "MOVED_PERMANENTLY"),
    (302, "FOUND"),
    (303, "SEE_OTHER"),
    (304, "NOT_MODIFIED"),
    (305, "USE_PROXY"),
    (307, "TEMPORARY_REDIRECT"),
    (308, "PERMANENT_REDIRECT"),
    (400, "BAD_REQUEST"),
    (401, "UNAUTHORIZED"),
    (402, "PAYMENT_REQUIRED"),
    (403, "FORBIDDEN"),
    (404, "NOT_FOUND"),
    (405, "METHOD_NOT_ALLOWED"),
    (406, "NOT_ACCEPTABLE"),
    (407, "PROXY_AUTHENTICATION_REQUIRED"),
    (408, "REQUEST_TIMEOUT"),
    (409, "CONFLICT"),
    (410, "GONE"),
    (411, "LENGTH_REQUIRED"),
    (412, "PRECONDITION_FAILED"),
    (413, "PAYLOAD_TOO_LARGE"),
    (414, "URI_TOO_LONG"),
    (415, "UNSUPPORTED_MEDIA_TYPE"),
    (416, "RANGE_NOT_SATISFIABLE"),
    (417, "EXPECTATION_FAILED"),
    (418, "IM_A_TEAPOT"),
    (421, "MISDIRECTED_REQUEST"),
    (422, "UNPROCESSABLE_ENTITY"),
    (423, "LOCKED"),
    (424, "FAILED_DEPENDENCY"),
    (426, "UPGRADE_REQUIRED"),
    (428, "PRECONDITION_REQUIRED"),
    (429, "TOO_MANY_REQUESTS"),
    (431, "REQUEST_HEADER_FIELDS_TOO_LARGE"),
    (451, "UNAVAILABLE_FOR_LEGAL_REASONS"),
    (500, "INTERNAL_SERVER_ERROR"),
    (501, "NOT_IMPLEMENTED"),
    (502, "BAD_GATEWAY"),
    (503, "SERVICE_UNAVAILABLE"),
    (504, "GATEWAY_TIMEOUT"),
    (505, "HTTP_VERSION_NOT_SUPPORTED"),
    (506, "VARIANT_ALSO_NEGOTIATES"),
    (507, "INSUFFICIENT_STORAGE"),
    (508, "LOOP_DETECTED"),
    (510, "NOT_EXTENDED"),
    (511, "NETWORK_AUTHENTICATION_REQUIRED"),
];

#[derive(Clone)]
pub struct StatusCode {
    code: &'static str,
    span: Span,
}

impl Default for StatusCode {
    fn default() -> Self {
        StatusCode {
            code: "OK",
            span: Span::call_site(),
        }
    }
}

impl From<syn::Lit> for StatusCode {
    fn from(literal: syn::Lit) -> StatusCode {
        match literal {
            syn::Lit::Str(s) => StatusCode::from(s),
            syn::Lit::Int(i) => StatusCode::from(i),
            _ => panic!("unavailable literal type"),
        }
    }
}

impl From<syn::LitInt> for StatusCode {
    fn from(literal: syn::LitInt) -> StatusCode {
        let n = match literal.suffix() {
            syn::IntSuffix::U16 => literal.value() as u16,
            syn::IntSuffix::None => {
                let n = literal.value();
                if n > u16::max_value() as u64 {
                    panic!("The value of status code is out of range");
                }
                n as u16
            }
            _ => panic!("Unsupported type for status code"),
        };
        let &(_, code) = SUPPORTED_STATUSES
            .into_iter()
            .find(|&&(c, _)| c == n)
            .unwrap();
        StatusCode {
            code,
            span: literal.span(),
        }
    }
}

impl From<syn::LitStr> for StatusCode {
    fn from(literal: syn::LitStr) -> StatusCode {
        let value = literal.value();
        let &(_, code) = SUPPORTED_STATUSES
            .iter()
            .find(|&&(_, s)| s == value)
            .unwrap();
        StatusCode {
            code,
            span: literal.span(),
        }
    }
}

impl fmt::Debug for StatusCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("StatusCode").field(&self.code).finish()
    }
}

impl ToTokens for StatusCode {
    fn to_tokens(&self, tokens: &mut Tokens) {
        tokens.append(TokenTree::Term(Term::new(self.code, self.span)));
    }
}

#[derive(Debug)]
struct Variant {
    ident: Ident,
    kind: FieldKind,
    status_code: Option<StatusCode>,
}

#[derive(Debug)]
enum FieldKind {
    Unit,
    Named,
    Unnamed,
}

#[derive(Debug)]
enum Body {
    Struct(Option<StatusCode>),
    Enum {
        status_code: Option<StatusCode>,
        variants: Vec<Variant>,
    },
}

impl Body {
    pub fn from_data(data: syn::Data, attrs: Vec<syn::Attribute>) -> Self {
        let status_code = parse_status_code(&attrs);
        match data {
            syn::Data::Struct(..) => Body::Struct(status_code),
            syn::Data::Enum(data) => {
                let mut variants = vec![];
                for variant in data.variants {
                    let status_code = parse_status_code(&variant.attrs);
                    let kind = match variant.fields {
                        syn::Fields::Unit => FieldKind::Unit,
                        syn::Fields::Named(..) => FieldKind::Named,
                        syn::Fields::Unnamed(..) => FieldKind::Unnamed,
                    };
                    variants.push(Variant {
                        ident: variant.ident,
                        kind,
                        status_code,
                    });
                }
                Body::Enum {
                    status_code,
                    variants,
                }
            }
            syn::Data::Union(..) => panic!("union does not supported"),
        }
    }

    pub fn to_tokens(&self, ident: &Ident) -> Tokens {
        match *self {
            Body::Struct(ref status_code) => {
                let status_code = status_code.clone().unwrap_or_default();
                quote! {
                    StatusCode::#status_code
                }
            }
            Body::Enum {
                ref status_code,
                ref variants,
            } => {
                let inner = variants.into_iter().map(|variant| {
                    let name = &variant.ident;
                    let args = match variant.kind {
                        FieldKind::Unit => quote!(),
                        FieldKind::Named => quote!({ .. }),
                        FieldKind::Unnamed => quote!((..)),
                    };
                    let status_code = (variant.status_code.as_ref())
                        .or(status_code.as_ref())
                        .cloned()
                        .unwrap_or_default();
                    quote! {
                        #ident :: #name #args => StatusCode::#status_code
                    }
                });
                quote! {
                    match *self {
                        #(#inner,)*
                    }
                }
            }
        }
    }
}

fn parse_status_code(attrs: &[syn::Attribute]) -> Option<StatusCode> {
    let mut status_code = None;
    for meta in attrs.iter().filter_map(|a| a.interpret_meta()) {
        match meta {
            Meta::NameValue(nv) => {
                if nv.ident != "status_code" {
                    panic!("supported only 'status_code = \"STATUS_CODE\"'");
                }
                status_code = Some(nv.lit.into());
            }
            _ => continue,
        }
    }
    status_code
}

struct Context {
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

impl Context {
    fn new(input: DeriveInput) -> Self {
        Context {
            ident: input.ident,
            generics: input.generics,
            body: Body::from_data(input.data, input.attrs),
        }
    }

    fn dummy_module_ident(&self) -> Ident {
        format!("__impl_http_response_for_{}", self.ident).into()
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
                extern crate finchers as _finchers;
                extern crate http as _http;
                use self::_finchers::output::HttpResponse;
                use self::_http::StatusCode;

                impl #impl_generics HttpResponse for #ident #ty_generics #where_clause {
                    fn status_code(&self) -> StatusCode {
                        #body
                    }
                }
            }
        });
    }
}

pub fn derive(input: TokenStream) -> TokenStream {
    let input = syn::parse2(input).unwrap();
    let context = Context::new(input);
    context.into_tokens().into()
}
