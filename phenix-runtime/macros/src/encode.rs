use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{parse_quote, punctuated::Punctuated, DataEnum, DataStruct, Expr, Ident};

use crate::util;

pub fn encode_struct(data: &DataStruct, name: Ident, is_exhaustive: bool) -> TokenStream2 {
    if !is_exhaustive {
        return util::non_exhaustive_not_supported(name.span(), util::TypeKind::Struct)
            .into_compile_error();
    }

    let mut body = TokenStream2::new();

    let optional_fields = util::get_optional_fields(&data.fields);

    if !optional_fields.is_empty() {
        let mut optional_list = Punctuated::<Expr, syn::token::Comma>::new();

        for field in optional_fields.iter() {
            optional_list.push(parse_quote!(self.#field.is_some()));
        }

        body.extend(quote! {
            ::phenix_runtime::base::bool::encode_many(&[#optional_list], writer)?;
        });
    }

    for (i, field) in data.fields.iter().enumerate() {
        let field_name = field.ident.clone().unwrap_or_else(|| parse_quote!(#i));
        let encode_field = if util::is_option_type(&field.ty) {
            quote! {
                if let ::std::option::Option::Some(#field_name) = self.#field_name {
                    #field_name.encode(writer)?;
                }
            }
        } else {
            quote!(self.#field_name.encode(writer)?;)
        };

        body.extend(encode_field);
    }

    body.extend(quote!(::std::result::Result::Ok(())));
    body
}

pub fn encode_enum(data: &DataEnum, name: Ident, is_exhaustive: bool) -> TokenStream2 {
    if !is_exhaustive {
        return util::non_exhaustive_not_supported(name.span(), util::TypeKind::Enum)
            .into_compile_error();
    }

    let mut body = TokenStream2::new();

    let mut match_body = TokenStream2::new();

    for (i, variant) in data.variants.iter().enumerate() {
        if variant.discriminant.is_some() {
            return syn::Error::new(
                variant.ident.span(),
                "explicit discriminants are not supported",
            )
            .into_compile_error();
        }

        let variant_pat = util::into_variant_pat(&name, variant, false);

        match_body.extend(quote!(#variant_pat => #i,));
    }

    body.extend(quote! {
        let discriminant = match self {
            #match_body
        };
        ::phenix_runtime::base::utils::encode_discriminant(discriminant, writer)?;
    });

    match_body = TokenStream2::new();

    for variant in data.variants.iter() {
        let variant_pat = util::into_variant_pat(&name, variant, true);
        match_body.extend(quote!(#variant_pat => ));

        syn::token::Brace {
            span: Span::call_site(),
        }
        .surround(&mut match_body, |match_body| match variant.fields {
            syn::Fields::Named(ref fields) => {
                for field in fields.named.iter() {
                    let field_name = field.ident.clone().unwrap();
                    match_body.extend(quote!(#field_name.encode(writer)?;));
                }
            }
            syn::Fields::Unnamed(ref fields) => {
                for field_name in (0..fields.unnamed.len()).map(util::unnamed_field_name) {
                    match_body.extend(quote!(#field_name.encode(writer)?;));
                }
            }
            syn::Fields::Unit => {}
        });
    }

    body.extend(quote! {
        match self {
            #match_body
        }
    });

    body.extend(quote!(::std::result::Result::Ok(())));
    body
}
