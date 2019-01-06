extern crate proc_macro;

use {
    proc_macro::TokenStream,
    proc_macro2::Span,
    quote::*,
    syn::{
        parse_macro_input, //
        DeriveInput,
        Ident,
        LitStr,
        Type,
    },
};

/// A procedural macro to define code that defines a type that
/// implements `ExtractPath` from the specified string literal.
///
/// This macro is used internally in `path!()`.
#[allow(nonstandard_style)]
#[proc_macro_derive(ExtractPath, attributes(path))]
pub fn ExtractPath(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let mut path: Option<LitStr> = None;
    for attr in &input.attrs {
        if attr.path.is_ident("path") {
            match attr.parse_meta() {
                Ok(syn::Meta::NameValue(meta)) => match meta.lit {
                    syn::Lit::Str(lit) => path = Some(lit),
                    _ => {
                        return syn::parse::Error::new_spanned(attr, "not a string literal")
                            .to_compile_error()
                            .into()
                    }
                },
                Ok(..) => {
                    return syn::parse::Error::new_spanned(
                        attr,
                        "the attribute must be a `#[path = \"..\"]`",
                    )
                    .to_compile_error()
                    .into()
                }
                Err(err) => return err.to_compile_error().into(),
            }
        }
    }
    let path = match path {
        Some(path) => path,
        None => {
            return syn::parse::Error::new_spanned(&input, "missing attribute: #[path \"/path\"]")
                .to_compile_error()
                .into()
        }
    };

    let path_value = path.value();
    let components = match parse_path(&path_value, &path) {
        Ok(components) => components,
        Err(err) => return err.to_compile_error().into(),
    };
    let components = &components; // anchored

    let Self_ = &input.ident;
    let ExtractPath: syn::Path = syn::parse_quote!(finchers::endpoint::syntax::ExtractPath);
    let ExtractPathError: syn::Path =
        syn::parse_quote!(finchers::endpoint::syntax::ExtractPathError);
    let FromEncodedStr: syn::Path =
        syn::parse_quote!(finchers::endpoint::syntax::encoded::FromEncodedStr);
    let PreflightContext: syn::Path = syn::parse_quote!(finchers::endpoint::PreflightContext);

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let mut where_clause = where_clause.cloned();
    for component in components {
        match component {
            Component::SingleParam(ty) | Component::CatchAllParam(ty) => {
                let predicates = &mut where_clause
                    .get_or_insert_with(|| syn::WhereClause {
                        where_token: Default::default(),
                        predicates: Default::default(),
                    })
                    .predicates;
                predicates.push(syn::parse_quote!(#ty: #FromEncodedStr));
                if !predicates.trailing_punct() {
                    predicates.push_punct(Default::default());
                }
            }
            _ => {}
        }
    }

    let output_types = components.into_iter().filter_map(|c| match c {
        Component::Static(..) => None,
        Component::SingleParam(ty) => Some(ty),
        Component::CatchAllParam(ty) => Some(ty),
    });

    let mut output_idents: Vec<syn::Ident> = vec![];
    let mut extracts: Vec<syn::Stmt> = vec![];
    for component in components {
        match component {
            Component::Static(s) => {
                extracts.push(syn::parse_quote! {
                    match cx.next() {
                        Some(s) if s == #s => (),
                        _ => return Err(#ExtractPathError::not_matched()),
                    }
                });
            }

            Component::SingleParam(ty) => {
                let ident = Ident::new(&format!("__x_{}", output_idents.len()), Span::call_site());
                extracts.push(syn::parse_quote! {
                    let #ident = match cx.next() {
                        Some(s) => <#ty as #FromEncodedStr>::from_encoded_str(s)
                            .map_err(#ExtractPathError::new)?,
                        None => return Err(#ExtractPathError::not_matched()),
                    };
                });
                output_idents.push(ident);
            }

            Component::CatchAllParam(ty) => {
                let ident = Ident::new(&format!("__x_{}", output_idents.len()), Span::call_site());
                extracts.push(syn::parse_quote! {
                    let #ident = {
                        let result = <#ty as #FromEncodedStr>::from_encoded_str(cx.remaining_path());
                        let _ = cx.by_ref().count();
                        result.map_err(#ExtractPathError::new)?
                    };
                });
                output_idents.push(ident);
            }
        }
    }

    TokenStream::from(quote! {
        impl #impl_generics #ExtractPath for #Self_ #ty_generics
        #where_clause
        {
            type Output = (#(#output_types,)*);

            fn extract(cx: &mut #PreflightContext<'_>) -> Result<Self::Output, #ExtractPathError> {
                #(#extracts)*
                Ok((#(#output_idents,)*))
            }
        }
    })
}

#[derive(Debug)]
enum Component<'a> {
    Static(&'a str),
    SingleParam(Type),
    CatchAllParam(Type),
}

fn parse_path<'s>(s: &'s str, lit: &LitStr) -> syn::parse::Result<Vec<Component<'s>>> {
    let s = s.trim();
    if s.is_empty() {
        return Err(syn::parse::Error::new_spanned(
            lit,
            "the path literal must not be empty",
        ));
    }

    if !s.starts_with('/') {
        return Err(syn::parse::Error::new_spanned(
            lit,
            "the path literal must start with a slash",
        ));
    }

    if s == "/" {
        return Ok(vec![]);
    }

    let mut components = vec![];
    let mut iter = s.split('/').skip(1).peekable();
    while let Some(segment) = iter.next() {
        if segment.is_empty() {
            if iter.peek().is_some() {
                return Err(syn::parse::Error::new_spanned(
                    lit,
                    "a path segment must not be empty",
                ));
            }
            break;
        }

        if segment.starts_with('<') {
            if !segment.ends_with('>') {
                return Err(syn::parse::Error::new_spanned(
                    lit,
                    "a segment that extracts a parameter must be end with '>'",
                ));
            }
            if !segment.is_ascii() {
                return Err(syn::parse::Error::new_spanned(
                    lit,
                    "non-ascii character(s) in the parameter position",
                ));
            }
            let ty_str = &segment[1..segment.len() - 1];

            if ty_str.starts_with("..") {
                let ty: syn::Type = syn::parse_str(&ty_str[2..]) //
                    .map_err(|e| syn::parse::Error::new_spanned(lit, e))?;
                components.push(Component::CatchAllParam(ty));

                if iter.peek().is_some() {
                    return Err(syn::parse::Error::new_spanned(
                        lit,
                        "the catch-all parameter must be at the end of path",
                    ));
                }

                break;
            } else {
                let ty: syn::Type = syn::parse_str(ty_str) //
                    .map_err(|e| syn::parse::Error::new_spanned(lit, e))?;
                components.push(Component::SingleParam(ty));
            }
        } else {
            components.push(Component::Static(segment));
        }
    }

    Ok(components)
}
