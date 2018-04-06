use proc_macro2::TokenStream;
use syn;

fn parse_data(data: syn::Data) -> Option<syn::Field> {
    match data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Unnamed(syn::FieldsUnnamed { ref unnamed, .. }),
            ..
        }) if unnamed.len() == 1 =>
        {
            let field = *unnamed.first().unwrap().value();
            Some(field.clone())
        }
        _ => None,
    }
}

pub fn derive(input: TokenStream) -> TokenStream {
    let input: syn::DeriveInput = syn::parse2(input).unwrap();
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let field = parse_data(input.data).expect("The derivation of FromSegment is supported only for an unnamed field.");

    let ident = &input.ident;
    let dummy_mod = syn::Ident::from(format!("__impl_from_segment_for_{}", ident));
    let ty = &field.ty;
    let where_clause = match where_clause {
        Some(tokens) => quote!(#tokens, #ty: FromSegment,),
        None => quote!(where #ty: FromSegment,),
    };

    let tokens = quote! {
        #[allow(non_snake_case)]
        mod #dummy_mod {
            extern crate finchers as _finchers;
            use self::_finchers::endpoint::path::{Segment, FromSegment};

            impl #impl_generics FromSegment for #ident #ty_generics #where_clause
            {
                type Err = <#ty as FromSegment>::Err;

                #[inline]
                fn from_segment(s: Segment) -> Result<Self, Self::Err> {
                    <#ty as FromSegment>::from_segment(s).map(#ident)
                }
            }
        }
    };
    tokens.into()
}
