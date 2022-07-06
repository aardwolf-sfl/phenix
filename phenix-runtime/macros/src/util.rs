use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use syn::{
    parse_quote, punctuated::Punctuated, DeriveInput, Fields, GenericParam, Generics, Ident,
    ItemStruct, Type, TypePath, Variant,
};

pub enum TypeKind {
    Struct,
    Enum,
}

pub fn is_exhaustive(input: &DeriveInput) -> bool {
    !input
        .attrs
        .iter()
        .any(|attr| attr.path.is_ident("non_exhaustive"))
}

pub fn is_exhaustive_struct(input: &ItemStruct) -> bool {
    !input
        .attrs
        .iter()
        .any(|attr| attr.path.is_ident("non_exhaustive"))
}

pub fn add_trait_bounds(mut generics: Generics, trait_ty: TypePath) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(parse_quote!(#trait_ty));
        }
    }

    generics
}

pub fn add_generic_param(mut generics: Generics, param: GenericParam) -> Generics {
    match param {
        GenericParam::Type(_) | GenericParam::Const(_) => generics.params.push(param),
        GenericParam::Lifetime(_) => {
            let after_last_lt = generics
                .params
                .iter()
                .enumerate()
                .rev()
                .find_map(|(i, p)| {
                    if matches!(p, GenericParam::Lifetime(_)) {
                        Some(i + 1)
                    } else {
                        None
                    }
                })
                .unwrap_or(0);

            generics.params.insert(after_last_lt, param);
        }
    }

    generics
}

pub fn get_optional_fields(fields: &Fields) -> Vec<Ident> {
    fields
        .iter()
        .enumerate()
        .filter(|(_, field)| is_option_type(&field.ty))
        .map(|(i, field)| field.ident.clone().unwrap_or_else(|| parse_quote!(#i)))
        .collect()
}

pub fn unwrap_option_type(ty: &Type) -> Option<&Type> {
    let full_path = ["std", "option", "Option"];

    let path_ty = match ty {
        Type::Path(path_ty)
            if path_ty
                .path
                .segments
                .iter()
                .rev()
                .zip(full_path.into_iter().rev())
                .all(|(a, b)| a.ident == b) =>
        {
            path_ty
        }
        _ => return None,
    };

    match path_ty.path.segments.last()?.arguments {
        syn::PathArguments::AngleBracketed(ref arguments) => match arguments.args.first()? {
            syn::GenericArgument::Type(ty) => Some(ty),
            _ => None,
        },
        _ => None,
    }
}

pub fn is_option_type(ty: &Type) -> bool {
    unwrap_option_type(ty).is_some()
}

pub fn into_variant_pat(ty_name: &Ident, variant: &Variant, field_names: bool) -> TokenStream2 {
    let mut pat = TokenStream2::new();

    let variant_name = variant.ident.clone();
    pat.extend(quote!(#ty_name::#variant_name));

    if field_names {
        let fields_list: Punctuated<Ident, syn::token::Comma> = variant
            .fields
            .iter()
            .enumerate()
            .map(|(i, field)| field.ident.clone().unwrap_or_else(|| unnamed_field_name(i)))
            .collect();

        match variant.fields {
            Fields::Named(ref named) => named
                .brace_token
                .surround(&mut pat, |pat| pat.extend(fields_list.into_token_stream())),
            Fields::Unnamed(ref unnamed) => unnamed
                .paren_token
                .surround(&mut pat, |pat| pat.extend(fields_list.into_token_stream())),
            Fields::Unit => {}
        }
    } else {
        // let mut fields_list = Punctuated::<Ident, syn::token::Comma>::new();

        match variant.fields {
            Fields::Named(_) => pat.extend(quote!({ .. })),
            Fields::Unnamed(ref fields) => fields.paren_token.surround(&mut pat, |pat| {
                pat.extend(
                    fields
                        .unnamed
                        .iter()
                        .map(|_| quote!(_))
                        .collect::<Punctuated<_, syn::token::Comma>>()
                        .into_token_stream(),
                )
            }),
            Fields::Unit => {}
        }
    }

    pat
}

pub fn unnamed_field_name(i: usize) -> Ident {
    Ident::new(&format!("f{}", i), Span::call_site())
}

pub fn non_exhaustive_not_supported(span: Span, ty_kind: TypeKind) -> syn::Error {
    let ty_kind = match ty_kind {
        TypeKind::Struct => "structs",
        TypeKind::Enum => "enums",
    };
    syn::Error::new(
        span,
        format!("non exhaustive {} are not yet supported", ty_kind),
    )
}

pub fn unions_not_supported(span: Span) -> syn::Error {
    syn::Error::new(span, "unions are not supported")
}
