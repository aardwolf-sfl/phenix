use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{punctuated::Punctuated, DataEnum, DataStruct, Ident};

use crate::util;

pub fn decode_struct(data: &DataStruct, name: Ident, is_exhaustive: bool) -> TokenStream2 {
    if !is_exhaustive {
        return util::non_exhaustive_not_supported(name.span(), util::TypeKind::Struct)
            .into_compile_error();
    }

    let mut body = TokenStream2::new();

    let optional_fields = util::get_optional_fields(&data.fields);

    if !optional_fields.is_empty() {
        let count = optional_fields.len();

        body.extend(quote! {
            let optional__ = ::phenix_runtime::base::bool::recognize_many(bytes, #count)?;
            let optional__ = optional__.as_bytes();
        });
    }

    let mut optional_bit = 0usize;

    for (i, field) in data.fields.iter().enumerate() {
        let field_name = field
            .ident
            .clone()
            .unwrap_or_else(|| util::unnamed_field_name(i));
        let field_ty = &field.ty;

        let decode_field = match util::unwrap_option_type(&field.ty) {
            Some(option_ty) => {
                let decode_field = quote! {
                    if ::phenix_runtime::base::utils::test_bit_at(#optional_bit, optional__) {
                        ::std::option::Option::Some(<#option_ty>::decode(bytes, buf)?)
                    } else {
                        ::std::option::Option::None
                    }
                };

                optional_bit += 1;
                decode_field
            }
            None => quote!(<#field_ty>::decode(bytes, buf)?),
        };

        body.extend(quote!( let #field_name = #decode_field;));
    }

    let fields = data.fields.iter().enumerate().map(|(i, field)| {
        field
            .ident
            .clone()
            .unwrap_or_else(|| util::unnamed_field_name(i))
    });

    body.extend(quote! {
        ::std::result::Result::Ok(Self {
            #(#fields,)*
        })
    });
    body
}

pub fn decode_enum(data: &DataEnum, name: Ident, is_exhaustive: bool) -> TokenStream2 {
    if !is_exhaustive {
        return util::non_exhaustive_not_supported(name.span(), util::TypeKind::Enum)
            .into_compile_error();
    }

    let mut body = TokenStream2::new();

    body.extend(
        quote!(let discriminant = ::phenix_runtime::base::utils::decode_items_count(bytes)?;),
    );

    let mut match_body = TokenStream2::new();

    for (i, variant) in data.variants.iter().enumerate() {
        if variant.discriminant.is_some() {
            return syn::Error::new(
                variant.ident.span(),
                "explicit discriminants are not supported",
            )
            .into_compile_error();
        }

        let variant_name = variant.ident.clone();

        let decode_fields = variant.fields.iter().enumerate().map(|(i, field)| {
            let field_name = field
                .ident
                .clone()
                .unwrap_or_else(|| util::unnamed_field_name(i));
            let field_ty = &field.ty;

            quote!(let #field_name = <#field_ty>::decode(bytes, buf)?;)
        });

        let fields_list = variant
            .fields
            .iter()
            .enumerate()
            .map(|(i, field)| {
                field
                    .ident
                    .clone()
                    .unwrap_or_else(|| util::unnamed_field_name(i))
            })
            .collect::<Punctuated<Ident, syn::token::Comma>>()
            .into_token_stream();

        let mut initialize = TokenStream2::new();

        match variant.fields {
            syn::Fields::Named(ref named) => named
                .brace_token
                .surround(&mut initialize, |ts| ts.extend(fields_list)),
            syn::Fields::Unnamed(ref unnamed) => unnamed
                .paren_token
                .surround(&mut initialize, |ts| ts.extend(fields_list)),
            syn::Fields::Unit => {}
        };

        match_body.extend(quote! {
            #i => {
                #(#decode_fields)*
                #name::#variant_name #initialize
            }
        });
    }

    match_body
        .extend(quote!(_ => return ::std::result::Result::Err(::phenix_runtime::InvalidPrefix::new(bytes).into())));

    body.extend(quote! {
        let value = match discriminant {
            #match_body
        };

        ::std::result::Result::Ok(value)
    });
    body
}
