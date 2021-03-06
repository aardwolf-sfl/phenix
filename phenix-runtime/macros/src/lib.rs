use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, parse_quote, DeriveInput, ItemStruct};

mod decode;
mod encode;
mod parts;
mod recognize;
mod util;

#[proc_macro_derive(Encodable)]
pub fn encodable(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as DeriveInput);
    let is_exhaustive = util::is_exhaustive(&input);

    let name = input.ident;

    let generics =
        util::add_trait_bounds(input.generics, parse_quote!(::phenix_runtime::Encodable));
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let mut encode_body = TokenStream2::new();

    match input.data {
        syn::Data::Struct(data) => {
            encode_body.extend(encode::encode_struct(&data, name.clone(), is_exhaustive));
        }
        syn::Data::Enum(data) => {
            encode_body.extend(encode::encode_enum(&data, name.clone(), is_exhaustive));
        }
        syn::Data::Union(_) => {
            return util::unions_not_supported(name.span())
                .into_compile_error()
                .into();
        }
    }

    let expanded = quote! {
        impl #impl_generics phenix_runtime::Encodable for #name #ty_generics #where_clause {
            fn encode<W: ::std::io::Write>(&self, writer: &mut W) -> ::std::io::Result<()> {
                #encode_body
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(Decodable)]
pub fn decodable(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as DeriveInput);
    let is_exhaustive = util::is_exhaustive(&input);

    let name = input.ident;

    let generics =
        util::add_trait_bounds(input.generics, parse_quote!(::phenix_runtime::Decodable));
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let mut decode_body = TokenStream2::new();
    let mut recognize_body = TokenStream2::new();

    match input.data {
        syn::Data::Struct(data) => {
            decode_body.extend(decode::decode_struct(&data, name.clone(), is_exhaustive));
            recognize_body.extend(recognize::recognize_struct(
                &data,
                name.clone(),
                is_exhaustive,
            ));
        }
        syn::Data::Enum(data) => {
            decode_body.extend(decode::decode_enum(&data, name.clone(), is_exhaustive));
            recognize_body.extend(recognize::recognize_enum(
                &data,
                name.clone(),
                is_exhaustive,
            ));
        }
        syn::Data::Union(_) => {
            return util::unions_not_supported(name.span())
                .into_compile_error()
                .into();
        }
    }

    let expanded = quote! {
        impl #impl_generics ::phenix_runtime::Decodable for #name #ty_generics #where_clause {
            fn decode(
                bytes: &mut ::phenix_runtime::bytes::Bytes<'_>,
            ) -> ::std::result::Result<Self, ::phenix_runtime::DecodingError> {
                #decode_body
            }

            fn recognize<'a>(
                bytes: &mut ::phenix_runtime::bytes::Bytes<'a>,
            ) -> ::std::result::Result<::phenix_runtime::bytes::ByteSlice<'a, Self>, ::phenix_runtime::DecodingError> {
                #recognize_body
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(IsFlag)]
pub fn is_flag(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as DeriveInput);
    let is_exhaustive = util::is_exhaustive(&input);

    let name = input.ident;

    let data = match input.data {
        syn::Data::Enum(data)
            if data
                .variants
                .iter()
                .all(|variant| matches!(variant.fields, syn::Fields::Unit)) =>
        {
            data
        }
        _ => {
            return syn::Error::new(name.span(), "IsFlag is only supported for bare enums")
                .into_compile_error()
                .into();
        }
    };

    let flags = data
        .variants
        .iter()
        .map(|variant| {
            let variant_name = &variant.ident;
            quote!(#name::#variant_name)
        })
        .collect::<Vec<_>>();

    let flags_bit_index = flags
        .iter()
        .enumerate()
        .map(|(i, flag)| quote!(#flag => #i));

    let count = flags.len();

    let expanded = quote! {
        impl ::phenix_runtime::IsFlag for #name {
            type IntoIter = ::std::array::IntoIter<Self, #count>;

            const COUNT: usize = #count;
            const IS_EXHAUSTIVE: bool = #is_exhaustive;

            fn bit_index(&self) -> usize {
                match self {
                    #(#flags_bit_index,)*
                }
            }

            fn all() -> Self::IntoIter {
                [#(#flags,)*].into_iter()
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn by_parts(_: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);
    let is_exhaustive = util::is_exhaustive_struct(&input);

    let item = input.clone();

    let name = input.ident.clone();
    let generics = input.generics.clone();
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let (parts_name, lifetime, parts_generics) = parts::prepare(&input);
    let (_, parts_ty_generics, parts_where_clause) = parts_generics.split_for_impl();

    let parts = input.fields.iter().enumerate().map(|(i, field)| {
        let variant = parts::variant_name(i, field);
        let ty = &field.ty;

        let data = match util::unwrap_option_type(ty) {
            Some(optional) => quote!(::std::option::Option<::phenix_runtime::bytes::ByteSlice<#lifetime, #optional>>),
            None => quote!(::phenix_runtime::bytes::ByteSlice<#lifetime, #ty>),
        };

        quote!(#variant(#data))
    });

    let recognize_by_parts = parts::impl_recognize_by_parts(&item, is_exhaustive);

    let expanded = quote! {
        #item

        #[derive(Debug)]
        pub enum #parts_name #parts_ty_generics #parts_where_clause {
            #(#parts),*
        }

        impl #impl_generics #name #ty_generics #where_clause {
            pub fn recognize_by_parts<'input, #lifetime>(
                bytes: &'input mut ::phenix_runtime::bytes::Bytes<#lifetime>,
            ) -> impl ::std::iter::Iterator<Item = ::std::result::Result<#parts_name #parts_ty_generics, ::phenix_runtime::DecodingError>> + 'input
            where
                Self: ::phenix_runtime::Decodable
            {
                #recognize_by_parts
            }
        }
    };

    TokenStream::from(expanded)
}
