//! Implementation of the `FromAnsi` derive macro.
//!
//! This module generates implementations of the `TryFromAnsi` trait for both
//! enums and structs.
//!
//! # Enum Support
//!
//! For enums, it supports two parsing strategies:
//!
//! 1. **Primitive representation** - for enums with `#[repr(u8)]` and similar
//!    attributes, parsing from integer values
//! 2. **String-based** - for enums without repr, parsing from string values
//!    via `TryFrom<&str>`
//!
//! Both strategies support optional default variants that can either return a
//! constant value or capture the unparsed input when parsing fails.
//!
//! # Struct Support
//!
//! For structs, the module generates parameter decoding based on format
//! attributes (`map` or `vector`) and handles both named and tuple structs.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields};

use crate::helpers::type_props::ValueProperties;
use crate::helpers::{
    DefaultVariant, HasTypeProperties, extract_struct_param_info,
    extract_vec_inner_type, find_default_variant, generate_doc_imports,
    get_primary_lifetime, metadata::FieldLocation, metadata::StructFormat,
    non_enum_error, non_struct_error,
};

use crate::macros::param_decoder::{
    ParamDecodingContext, ParamSource, ParamSourceFormat,
    generate_param_decoding,
};

/// Generate the implementation of `TryFromAnsi` for an enum or struct.
///
/// This function dispatches to the appropriate generator based on whether the
/// input is an enum or a struct.
///
/// # Errors
///
/// Return an error if:
/// - The input is neither an enum nor a struct with named fields
/// - The attributes cannot be parsed
/// - The configuration is invalid
pub fn from_ansi_inner(ast: &DeriveInput) -> syn::Result<TokenStream> {
    // Generate doc imports for IDE hover support
    let doc_imports = generate_doc_imports(ast);

    let impl_code = match &ast.data {
        Data::Enum(_) => generate_enum_impl(ast),
        Data::Struct(data) => match &data.fields {
            Fields::Named(_) | Fields::Unnamed(_) => generate_struct_impl(ast),
            Fields::Unit => Err(syn::Error::new_spanned(
                ast,
                "FromAnsi cannot be derived for unit structs",
            )),
        },
        Data::Union(_) => Err(syn::Error::new_spanned(
            ast,
            "FromAnsi cannot be derived for unions",
        )),
    }?;

    Ok(quote! {
        #doc_imports
        #impl_code
    })
}

/// Generate the implementation of `TryFromAnsi` for a struct.
///
/// This function generates parsing code based on the struct's format
/// attribute (`map` or `vector`) and delimiter. For tuple structs,
/// it automatically uses `value` format.
///
/// # Errors
///
/// Return an error if:
/// - The input is not a struct with named or unnamed fields
/// - The attributes cannot be parsed
/// - Fields have invalid configurations
/// - `map` format is used with tuple structs
pub fn generate_struct_impl(ast: &DeriveInput) -> syn::Result<TokenStream> {
    let props = ast.get_type_properties()?;
    if props.transparent {
        generate_transparent_struct_impl(ast, &props)
    } else {
        generate_normal_struct_impl(ast, &props)
    }
}

pub fn generate_normal_struct_impl(
    ast: &DeriveInput,
    props: &ValueProperties,
) -> syn::Result<TokenStream> {
    let Data::Struct(_) = &ast.data else {
        return Err(non_struct_error());
    };

    let name = &ast.ident;
    let generics = &ast.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let (lt, trait_lt) = if let Some(lt) = get_primary_lifetime(ast) {
        (quote! { #lt }, quote! { <#lt> })
    } else {
        (quote! {}, quote! { <'_> })
    };

    // Handle normal structs - generate TryFromAnsi implementation
    let source_ident =
        syn::Ident::new("__vtansi_data", proc_macro2::Span::mixed_site());
    let stype: syn::Type = syn::parse_quote!(Self);
    let params = extract_struct_param_info(ast, None, FieldLocation::Params)?;
    let source = ParamSource::new(&source_ident, ParamSourceFormat::Flat);
    let ctx = ParamDecodingContext::new(&stype, &params, &props, &source)
        .with_into(props.into.as_ref());
    let (param_decoding, constructor) = generate_param_decoding(&ctx)?;

    // For TryFromAnsiIter, we need a lifetime parameter for the trait
    let iter_lt = if get_primary_lifetime(ast).is_some() {
        quote! { #lt }
    } else {
        quote! { '__vtansi_iter_lt }
    };

    let iter_impl_generics = if get_primary_lifetime(ast).is_some() {
        quote! { #impl_generics }
    } else {
        // Insert a lifetime parameter for the trait bound
        let gparams = &generics.params;
        if gparams.is_empty() {
            quote! { <'__vtansi_iter_lt> }
        } else {
            quote! { <'__vtansi_iter_lt, #gparams> }
        }
    };

    let iter_where_clause = quote! { #where_clause };

    // Generate TryFromAnsiIter implementation based on format
    // Note: Map format does not get automatic TryFromAnsiIter - it must be implemented manually
    let iter_impl = match props.format {
        StructFormat::Map => {
            // No automatic TryFromAnsiIter for map format
            quote! {}
        }
        StructFormat::Vector => {
            // For vector format, consume one item per field
            let iter_source_ident = syn::Ident::new(
                "__vtansi_iter",
                proc_macro2::Span::call_site(),
            );
            let iter_source =
                ParamSource::new(&iter_source_ident, ParamSourceFormat::Iter);
            let iter_ctx = ParamDecodingContext::new(
                &stype,
                &params,
                &props,
                &iter_source,
            )
            .with_into(props.into.as_ref());
            let (iter_param_decoding, iter_constructor) =
                generate_param_decoding(&iter_ctx)?;

            quote! {
                #[allow(clippy::use_self)]
                #[automatically_derived]
                impl #iter_impl_generics ::vtansi::parse::TryFromAnsiIter<#iter_lt> for #name #ty_generics #iter_where_clause {
                    #[inline]
                    fn try_from_ansi_iter<__VtansiIter>(__vtansi_iter: &mut __VtansiIter) -> ::core::result::Result<Self, ::vtansi::parse::ParseError>
                    where
                        __VtansiIter: ::core::iter::Iterator<Item = &#iter_lt [u8]>,
                    {
                        ::core::result::Result::Ok({
                            #iter_param_decoding
                            #iter_constructor
                        })
                    }
                }
            }
        }
    };

    Ok(quote! {
        #[allow(clippy::use_self)]
        #[automatically_derived]
        impl #impl_generics ::vtansi::parse::TryFromAnsi #trait_lt for #name #ty_generics #where_clause {
            #[inline]
            fn try_from_ansi(#source_ident: &#lt [u8]) -> ::core::result::Result<Self, ::vtansi::parse::ParseError> {
                ::core::result::Result::Ok({
                    #param_decoding
                    #constructor
                })
            }
        }

        #iter_impl
    })
}

pub fn generate_transparent_struct_impl(
    ast: &DeriveInput,
    props: &ValueProperties,
) -> syn::Result<TokenStream> {
    let Data::Struct(data) = &ast.data else {
        return Err(non_struct_error());
    };

    let name = &ast.ident;
    let generics = &ast.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let (lt, trait_lt) = if let Some(lt) = get_primary_lifetime(ast) {
        (quote! { #lt }, quote! { <#lt> })
    } else {
        (quote! {}, quote! { <'_> })
    };

    let fields = match &data.fields {
        Fields::Named(fields) => fields.named.iter().collect::<Vec<_>>(),
        Fields::Unnamed(fields) => fields.unnamed.iter().collect::<Vec<_>>(),
        Fields::Unit => Vec::new(),
    };

    if fields.len() != 1 {
        return Err(syn::Error::new_spanned(
            ast,
            format!(
                "transparent structs must have exactly one field, found {}",
                fields.len()
            ),
        ));
    }

    let field = fields[0];
    let field_ty = &field.ty;
    let constructor = match &field.ident {
        Some(ident) => quote! { Self { #ident: value } },
        None => quote! { Self(value) },
    };

    // For TryFromAnsiIter, we need a lifetime parameter for the trait
    let iter_lt = if get_primary_lifetime(ast).is_some() {
        quote! { #lt }
    } else {
        quote! { '__vtansi_iter_lt }
    };

    let iter_impl_generics = if get_primary_lifetime(ast).is_some() {
        quote! { #impl_generics }
    } else {
        // Insert a lifetime parameter for the trait bound
        let params = &generics.params;
        if params.is_empty() {
            quote! { <'__vtansi_iter_lt> }
        } else {
            quote! { <'__vtansi_iter_lt, #params> }
        }
    };

    // Check if the field type is Vec<T>
    if let Some(inner_ty) = extract_vec_inner_type(field_ty) {
        // Vec type requires delimiter attribute
        let delimiter = &props.delimiter.to_literal();

        Ok(quote! {
            #[allow(clippy::use_self)]
            #[automatically_derived]
            impl #impl_generics ::vtansi::parse::TryFromAnsi #trait_lt for #name #ty_generics #where_clause {
                #[inline]
                fn try_from_ansi(bytes: &[u8]) -> ::core::result::Result<Self, ::vtansi::parse::ParseError> {
                    let delimiter = #delimiter;
                    let value: ::std::vec::Vec<#inner_ty> = if bytes.is_empty() {
                        ::std::vec::Vec::new()
                    } else {
                        bytes
                            .split(|&b| b == delimiter)
                            .map(|part| <#inner_ty as ::vtansi::parse::TryFromAnsi>::try_from_ansi(part))
                            .collect::<::core::result::Result<::std::vec::Vec<_>, _>>()?
                    };
                    ::core::result::Result::Ok(#constructor)
                }
            }

            #[allow(clippy::use_self)]
            #[automatically_derived]
            impl #iter_impl_generics ::vtansi::parse::TryFromAnsiIter<#iter_lt> for #name #ty_generics #where_clause {
                #[inline]
                fn try_from_ansi_iter<__VtansiIter>(__vtansi_iter: &mut __VtansiIter) -> ::core::result::Result<Self, ::vtansi::parse::ParseError>
                where
                    __VtansiIter: ::core::iter::Iterator<Item = &#iter_lt [u8]>,
                {
                    // Consume all remaining items from the iterator
                    let value: ::std::vec::Vec<#inner_ty> = __vtansi_iter
                        .map(|part| <#inner_ty as ::vtansi::parse::TryFromAnsi>::try_from_ansi(part))
                        .collect::<::core::result::Result<::std::vec::Vec<_>, _>>()?;
                    ::core::result::Result::Ok(#constructor)
                }
            }
        })
    } else {
        // Non-Vec type: simple delegation
        Ok(quote! {
            #[allow(clippy::use_self)]
            #[automatically_derived]
            impl #impl_generics ::vtansi::parse::TryFromAnsi #trait_lt for #name #ty_generics #where_clause {
                #[inline]
                fn try_from_ansi(bytes: &[u8]) -> ::core::result::Result<Self, ::vtansi::parse::ParseError> {
                    let value = <#field_ty as ::vtansi::parse::TryFromAnsi>::try_from_ansi(bytes)?;
                    ::core::result::Result::Ok(#constructor)
                }
            }

            #[allow(clippy::use_self)]
            #[automatically_derived]
            impl #iter_impl_generics ::vtansi::parse::TryFromAnsiIter<#iter_lt> for #name #ty_generics #where_clause {
                #[inline]
                fn try_from_ansi_iter<__VtansiIter>(__vtansi_iter: &mut __VtansiIter) -> ::core::result::Result<Self, ::vtansi::parse::ParseError>
                where
                    __VtansiIter: ::core::iter::Iterator<Item = &#iter_lt [u8]>,
                {
                    // Consume one item and delegate to inner type's TryFromAnsi
                    let bytes = __vtansi_iter.next().ok_or(::vtansi::parse::ParseError::Empty)?;
                    let value = <#field_ty as ::vtansi::parse::TryFromAnsi>::try_from_ansi(bytes)?;
                    ::core::result::Result::Ok(#constructor)
                }
            }
        })
    }
}

/// Generate the implementation of `TryFromAnsi` for an enum.
///
/// This function orchestrates the code generation process by:
/// 1. Extracting type-level properties (e.g., repr type)
/// 2. Finding the default variant, if any
/// 3. Delegating to the appropriate generation function based on the repr
///    type
///
/// # Errors
///
/// Return an error if:
/// - The attributes cannot be parsed
/// - The default variant is invalid
fn generate_enum_impl(ast: &DeriveInput) -> syn::Result<TokenStream> {
    let name = &ast.ident;
    let generics = &ast.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let Data::Enum(enum_data) = &ast.data else {
        return Err(non_enum_error());
    };

    // Extract type-level properties
    let props = ast.get_type_properties()?;

    // Find default variant if any
    let default_variant = find_default_variant(enum_data)?;

    let expanded = if let Some(repr_type) = props.repr_type {
        // Generate implementation using the primitive representation
        generate_repr_impl(
            name,
            &impl_generics,
            &ty_generics,
            where_clause,
            &repr_type,
            default_variant,
        )
    } else {
        // Generate implementation using TryFrom<&str>
        generate_string_impl(
            name,
            &impl_generics,
            &ty_generics,
            where_clause,
            default_variant,
        )
    };

    Ok(expanded)
}

/// Generate implementation for enums with primitive repr.
///
/// This function creates a `TryFromAnsi` implementation that:
/// 1. Parses the input bytes as the primitive repr type
/// 2. Attempts to convert the number to the enum using `TryFrom`
/// 3. If a default variant is present, uses it on conversion failure
/// 4. Otherwise, returns an error on conversion failure
///
/// The implementation differs based on whether a default variant is present
/// and whether it's a unit or capturing variant.
fn generate_repr_impl(
    name: &syn::Ident,
    impl_generics: &syn::ImplGenerics,
    ty_generics: &syn::TypeGenerics,
    where_clause: Option<&syn::WhereClause>,
    repr_type: &syn::Ident,
    default_variant: Option<DefaultVariant>,
) -> TokenStream {
    match default_variant {
        Some(DefaultVariant::Unit(default_var)) => {
            quote! {
                #[allow(clippy::use_self)]
                #[automatically_derived]
                impl #impl_generics ::vtansi::parse::TryFromAnsi<'_> for #name #ty_generics #where_clause {
                    #[inline]
                    fn try_from_ansi(bytes: &[u8]) -> ::core::result::Result<Self, ::vtansi::parse::ParseError> {
                        use ::core::convert::TryFrom;

                        // Parse as the repr type
                        let num = <#repr_type as ::vtansi::parse::TryFromAnsi>::try_from_ansi(bytes)?;

                        // Convert to enum using TryFrom, or use default on
                        // failure
                        ::core::result::Result::Ok(Self::try_from(num).unwrap_or(Self::#default_var))
                    }
                }
            }
        }
        Some(DefaultVariant::Capturing(default_var)) => {
            quote! {
                #[allow(clippy::use_self)]
                #[automatically_derived]
                impl #impl_generics ::vtansi::parse::TryFromAnsi<'_> for #name #ty_generics #where_clause {
                    #[inline]
                    fn try_from_ansi(bytes: &[u8]) -> ::core::result::Result<Self, ::vtansi::parse::ParseError> {
                        use ::core::convert::TryFrom;

                        // Parse as the repr type
                        let num = <#repr_type as ::vtansi::parse::TryFromAnsi>::try_from_ansi(bytes)?;

                        // Convert to enum using TryFrom, or capture value in
                        // default on failure
                        ::core::result::Result::Ok(
                            Self::try_from(num).unwrap_or_else(|_| Self::#default_var(num.into()))
                        )
                    }
                }
            }
        }
        None => {
            quote! {
                #[allow(clippy::use_self)]
                #[automatically_derived]
                impl #impl_generics ::vtansi::parse::TryFromAnsi<'_> for #name #ty_generics #where_clause {
                    #[inline]
                    fn try_from_ansi(bytes: &[u8]) -> ::core::result::Result<Self, ::vtansi::parse::ParseError> {
                        use ::core::convert::TryFrom;

                        // Parse as the repr type
                        let num = <#repr_type as ::vtansi::parse::TryFromAnsi>::try_from_ansi(bytes)?;

                        // Convert to enum using TryFrom
                        Self::try_from(num).map_err(|_| {
                            ::vtansi::parse::ParseError::InvalidValue(
                                ::std::format!("invalid enum discriminant: {}", num)
                            )
                        })
                    }
                }
            }
        }
    }
}

/// Generate implementation for string-based enums.
///
/// This function creates a `TryFromAnsi` implementation that:
/// 1. Parses the input bytes as a UTF-8 string slice
/// 2. Attempts to convert the string to the enum using `TryFrom<&str>`
/// 3. If a default variant is present, uses it on conversion failure
/// 4. Otherwise, returns an error on conversion failure
///
/// The implementation differs based on whether a default variant is present
/// and whether it's a unit or capturing variant.
fn generate_string_impl(
    name: &syn::Ident,
    impl_generics: &syn::ImplGenerics,
    ty_generics: &syn::TypeGenerics,
    where_clause: Option<&syn::WhereClause>,
    default_variant: Option<DefaultVariant>,
) -> TokenStream {
    match default_variant {
        Some(DefaultVariant::Unit(default_var)) => {
            quote! {
                #[allow(clippy::use_self)]
                #[automatically_derived]
                impl #impl_generics ::vtansi::parse::TryFromAnsi<'_> for #name #ty_generics #where_clause {
                    #[inline]
                    fn try_from_ansi(bytes: &[u8]) -> ::core::result::Result<Self, ::vtansi::parse::ParseError> {
                        use ::core::convert::TryFrom;

                        // Parse as &str
                        let s = <&str as ::vtansi::parse::TryFromAnsi>::try_from_ansi(bytes)?;

                        // Convert to enum using TryFrom<&str>, or use default
                        // on failure
                        ::core::result::Result::Ok(Self::try_from(s).unwrap_or(Self::#default_var))
                    }
                }
            }
        }
        Some(DefaultVariant::Capturing(default_var)) => {
            quote! {
                #[allow(clippy::use_self)]
                #[automatically_derived]
                impl #impl_generics ::vtansi::parse::TryFromAnsi<'_> for #name #ty_generics #where_clause {
                    #[inline]
                    fn try_from_ansi(bytes: &[u8]) -> ::core::result::Result<Self, ::vtansi::parse::ParseError> {
                        use ::core::convert::TryFrom;

                        // Parse as &str
                        let s = <&str as ::vtansi::parse::TryFromAnsi>::try_from_ansi(bytes)?;

                        // Convert to enum using TryFrom<&str>, or capture
                        // value in default on failure
                        ::core::result::Result::Ok(
                            Self::try_from(s).unwrap_or_else(|_| Self::#default_var(s.into()))
                        )
                    }
                }
            }
        }
        None => {
            quote! {
                #[allow(clippy::use_self)]
                #[automatically_derived]
                impl #impl_generics ::vtansi::parse::TryFromAnsi<'_> for #name #ty_generics #where_clause {
                    #[inline]
                    fn try_from_ansi(bytes: &[u8]) -> ::core::result::Result<Self, ::vtansi::parse::ParseError> {
                        use ::core::convert::TryFrom;

                        // Parse as &str
                        let s = <&str as ::vtansi::parse::TryFromAnsi>::try_from_ansi(bytes)?;

                        // Convert to enum using TryFrom<&str>
                        Self::try_from(s).map_err(|_| {
                            ::vtansi::parse::ParseError::InvalidValue(
                                ::std::format!("invalid enum value: {}", s)
                            )
                        })
                    }
                }
            }
        }
    }
}
