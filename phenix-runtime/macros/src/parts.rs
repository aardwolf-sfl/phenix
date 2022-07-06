use convert_case::{Case, Casing};
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::{Field, Generics, ItemStruct, Lifetime};

use crate::util;

pub fn prepare(item: &ItemStruct) -> (Ident, Lifetime, Generics) {
    let parts_name = format_ident!("{}Part", item.ident);

    let lifetime = syn::Lifetime::new("'bytes", proc_macro2::Span::call_site());
    let parts_generics = util::add_generic_param(
        item.generics.clone(),
        syn::GenericParam::Lifetime(syn::LifetimeDef::new(lifetime.clone())),
    );

    (parts_name, lifetime, parts_generics)
}

pub fn variant_name(i: usize, field: &Field) -> Ident {
    let name = field
        .ident
        .clone()
        .unwrap_or_else(|| util::unnamed_field_name(i));

    format_ident!("{}", name.to_string().to_case(Case::Pascal))
}

pub fn impl_recognize_by_parts(item: &ItemStruct, is_exhaustive: bool) -> TokenStream2 {
    if !is_exhaustive {
        return util::non_exhaustive_not_supported(item.ident.span(), util::TypeKind::Struct)
            .into_compile_error();
    }

    let (parts_name, lifetime, parts_generics) = prepare(item);
    let (_, parts_ty_generics, _) = parts_generics.split_for_impl();

    let by_parts = format_ident!("{}ByParts", item.ident);
    let state = format_ident!("{}State", item.ident);

    let final_state = Ident::new("T", Span::call_site());
    let error_state = Ident::new("E", Span::call_site());

    let states = item
        .fields
        .iter()
        .enumerate()
        .map(|(i, _)| format_ident!("S{}", i))
        .collect::<Vec<_>>();

    let initial_state = states.first().unwrap().clone();

    let transitions = states
        .iter()
        .zip(states.iter().skip(1).chain(std::iter::once(&final_state)))
        .map(|(from, to)| quote!(#state :: #from => #state :: #to));

    let mut optional_bit = 0usize;

    let mut recognizers = Vec::new();

    for (s, (i, field)) in states.iter().zip(item.fields.iter().enumerate()) {
        let variant = variant_name(i, field);
        let field_ty = &field.ty;

        let recognize_field = match util::unwrap_option_type(&field.ty) {
            Some(option_ty) => {
                let recognize_field = quote! {
                    #state :: #s => if ::phenix_runtime::base::utils::test_bit_at(#optional_bit, optional) {
                        <#option_ty>::recognize(bytes).map(::std::option::Option::Some).map(#parts_name :: #variant)
                    } else {
                        ::std::result::Result::Ok(::std::option::Option::None).map(#parts_name :: #variant)
                    }
                };

                optional_bit += 1;
                recognize_field
            }
            None => {
                quote!(#state :: #s => <#field_ty>::recognize(bytes).map(#parts_name :: #variant))
            }
        };

        recognizers.push(recognize_field);
    }

    let mut body = TokenStream2::new();

    body.extend(quote! {
        #[derive(Clone, Copy, PartialEq)]
        enum #state {
            #(#states,)*
            #final_state,
            #error_state,
        }

        impl #state {
            fn next(self) -> Self {
                match self {
                    #(#transitions,)*
                    #state :: #final_state => #state :: #final_state,
                    #state :: #error_state => #state :: #final_state,
                }
            }

            fn recognize<#lifetime>(
                &self,
                bytes: &mut ::phenix_runtime::bytes::Bytes<#lifetime>,
                optional: &[u8],
                error: &mut ::std::option::Option<::phenix_runtime::DecodingError>,
            ) -> ::std::option::Option<::std::result::Result<#parts_name #parts_ty_generics, ::phenix_runtime::DecodingError>> {
                ::std::option::Option::Some(match self {
                    #(#recognizers,)*
                    #state :: #final_state => return ::std::option::Option::None,
                    #state :: #error_state => return error.take().map(Err),
                })
            }
        }

        struct #by_parts<#lifetime, 'input> {
            bytes: &'input mut ::phenix_runtime::bytes::Bytes<#lifetime>,
            optional: Vec<u8>,
            state: #state,
            error: Option<::phenix_runtime::DecodingError>,
        }

        impl<#lifetime, 'input> ::std::iter::Iterator for #by_parts<#lifetime, 'input> {
            type Item = ::std::result::Result<#parts_name #parts_ty_generics, ::phenix_runtime::DecodingError>;

            fn next(&mut self) -> ::std::option::Option<Self::Item> {
                let value = self.state.recognize(self.bytes, &self.optional, &mut self.error)?;
                self.state = self.state.next();
                Some(value)
            }
        }
    });

    let optional_fields = util::get_optional_fields(&item.fields);

    if !optional_fields.is_empty() {
        let count = optional_fields.len();

        body.extend(quote! {
            let (optional, state, error) = match ::phenix_runtime::base::bool::recognize_many(bytes, #count) {
                ::std::result::Result::Ok(optional) => (optional.as_bytes().to_vec(), #state :: #initial_state, ::std::option::Option::None),
                ::std::result::Result::Err(error) => (::std::vec::Vec::new(), #state :: #error_state, ::std::option::Option::Some(error)),
            };
        });
    } else {
        body.extend(quote! {
            let optional = ::std::vec::Vec::new();
            let state = #state :: #initial_state;
            let error = ::std::option::Option::None;
        });
    }

    body.extend(quote! {
        #by_parts {
            bytes,
            optional,
            state,
            error,
        }
    });
    body
}
