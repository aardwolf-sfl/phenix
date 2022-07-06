use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{DataEnum, DataStruct, Ident};

use crate::util;

pub fn recognize_struct(data: &DataStruct, name: Ident, is_exhaustive: bool) -> TokenStream2 {
    if !is_exhaustive {
        return util::non_exhaustive_not_supported(name.span(), util::TypeKind::Struct)
            .into_compile_error();
    }

    let mut body = TokenStream2::new();

    body.extend(quote!(let mark = bytes.mark();));

    let optional_fields = util::get_optional_fields(&data.fields);

    if !optional_fields.is_empty() {
        let count = optional_fields.len();

        body.extend(quote! {
            let optional = ::phenix_runtime::base::bool::recognize_many(bytes, #count)?;
            let optional = optional.as_bytes();
        });
    }

    let mut optional_bit = 0usize;

    for field in data.fields.iter() {
        let field_ty = &field.ty;

        let recognize_field = match util::unwrap_option_type(&field.ty) {
            Some(option_ty) => {
                let recognize_field = quote! {
                    if ::phenix_runtime::base::utils::test_bit_at(#optional_bit, optional) {
                        <#option_ty>::recognize(bytes)?;
                    }
                };

                optional_bit += 1;
                recognize_field
            }
            None => quote!(<#field_ty>::recognize(bytes)?;),
        };

        body.extend(recognize_field);
    }

    body.extend(quote!(::std::result::Result::Ok(
        bytes.take_slice_from(mark)
    )));
    body
}

pub fn recognize_enum(data: &DataEnum, name: Ident, is_exhaustive: bool) -> TokenStream2 {
    if !is_exhaustive {
        return util::non_exhaustive_not_supported(name.span(), util::TypeKind::Enum)
            .into_compile_error();
    }

    let mut body = TokenStream2::new();

    body.extend(quote!(let mark = bytes.mark();));

    body.extend(
        quote!(let discriminant = ::phenix_runtime::base::utils::decode_discriminant(bytes)?;),
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

        let recognize_fields = variant.fields.iter().map(|field| {
            let field_ty = &field.ty;
            quote!(<#field_ty>::recognize(bytes)?;)
        });

        match_body.extend(quote! {
            #i => {
                #(#recognize_fields)*
            }
        });
    }

    match_body
        .extend(quote!(_ => return ::std::result::Result::Err(::phenix_runtime::InvalidPrefix::new(bytes).into())));

    body.extend(quote! {
        match discriminant {
            #match_body
        }

        ::std::result::Result::Ok(bytes.take_slice_from(mark))
    });
    body
}
