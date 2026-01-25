//! Shared parameter decoding logic.
//!
//! This module provides reusable parameter decoding code generation that can
//! be used by both framed (control sequences) and unframed (plain struct)
//! decoding implementations.
//!
//! The key insight is that parameter decoding logic should be identical
//! whether the parameters appear in a control sequence (with intro/final
//! bytes) or as standalone struct values. This module extracts the common
//! decoding logic.

use std::collections::HashMap;

use proc_macro2::TokenStream;
use quote::quote;

use crate::helpers::{
    StructParamInfo, metadata::StructFormat, struct_params::Params,
    type_props::HasFormatProperties,
};

#[derive(Clone, Debug)]
pub enum ParamSourceFormat {
    /// Parameter data is a flat `&[u8]` slice that needs splitting
    Flat,
    /// Parameter data is already split, i.e a `&[&[u8]]`.
    Split,
    /// Parameter data comes from an external iterator (for TryFromAnsiIter)
    Iter,
}

#[derive(Clone, Debug)]
pub struct ParamSource {
    ident: syn::Ident,
    format: ParamSourceFormat,
}

impl ParamSource {
    pub fn new(ident: &syn::Ident, format: ParamSourceFormat) -> Self {
        Self {
            ident: ident.clone(),
            format,
        }
    }
}

/// Generate parameter decoding code for a list of fields.
///
/// This function generates code that parses parameters and constructs field
/// values. The generated code handles:
/// - Key-value format: fields can appear in any order, matched by name
/// - Value format: fields must appear in declaration order
/// - Optional fields (Option<T> types) - set to None if not present
/// - Required fields - error if not present
///
/// # Returns
///
/// A TokenStream containing the body suitable for implementing
/// `try_into_ansi()`.
#[allow(clippy::too_many_arguments)]
pub fn generate_param_decoding(
    typename: &syn::Type,
    params: &StructParamInfo,
    props: &impl HasFormatProperties,
    source: &ParamSource,
    static_params_source: Option<&ParamSource>,
    data_source: Option<&ParamSource>,
    finalbyte_source: Option<&ParamSource>,
    into: Option<&syn::Path>,
) -> syn::Result<(TokenStream, TokenStream)> {
    // Separate static_params fields from regular params fields
    let (static_params_fields, regular_params_fields): (Vec<_>, Vec<_>) =
        params
            .params
            .fields
            .iter()
            .partition(|f| f.is_static_params);

    // Create modified Params structs for each group
    let regular_params = Params {
        fields: regular_params_fields.into_iter().cloned().collect(),
        required_count: params
            .params
            .fields
            .iter()
            .filter(|f| !f.is_static_params && !f.is_optional)
            .count(),
        total_count: params
            .params
            .fields
            .iter()
            .filter(|f| {
                !f.is_static_params && f.mux_index.is_none() && !f.is_flatten
            })
            .count(),
        has_mux: params.params.has_mux,
        has_flatten: params.params.has_flatten,
        has_static_params: false,
    };

    let static_params = Params {
        fields: static_params_fields.into_iter().cloned().collect(),
        required_count: params
            .params
            .fields
            .iter()
            .filter(|f| f.is_static_params && !f.is_optional)
            .count(),
        total_count: params
            .params
            .fields
            .iter()
            .filter(|f| {
                f.is_static_params && f.mux_index.is_none() && !f.is_flatten
            })
            .count(),
        has_mux: false,
        has_flatten: false,
        has_static_params: true,
    };

    let decoding = match props.format() {
        StructFormat::Map => {
            let param_decoding =
                generate_map_decoding(&regular_params, props, source)?;
            let static_param_decoding = if let Some(static_source) =
                static_params_source
                && !static_params.is_empty()
            {
                generate_map_decoding(&static_params, props, static_source)?
            } else {
                quote! {}
            };
            let data_param_decoding = if let Some(data) = data_source
                && !params.data_params.is_empty()
            {
                generate_map_decoding(&params.data_params, props, data)?
            } else {
                quote! {}
            };
            let final_byte_decoding = if let Some(finalbyte_source) =
                finalbyte_source
                && !params.final_byte_params.is_empty()
            {
                generate_vector_decoding(
                    &params.final_byte_params,
                    props,
                    finalbyte_source,
                )?
            } else {
                quote! {}
            };

            quote! {
                #static_param_decoding
                #param_decoding
                #data_param_decoding
                #final_byte_decoding
            }
        }
        StructFormat::Vector => {
            let param_decoding =
                generate_vector_decoding(&regular_params, props, source)?;
            let static_param_decoding = if let Some(static_source) =
                static_params_source
                && !static_params.is_empty()
            {
                generate_vector_decoding(&static_params, props, static_source)?
            } else {
                quote! {}
            };
            let data_param_decoding = if let Some(data) = data_source
                && !params.data_params.is_empty()
            {
                generate_vector_decoding(&params.data_params, props, data)?
            } else {
                quote! {}
            };
            let final_byte_decoding = if let Some(finalbyte_source) =
                finalbyte_source
                && !params.final_byte_params.is_empty()
            {
                generate_vector_decoding(
                    &params.final_byte_params,
                    props,
                    finalbyte_source,
                )?
            } else {
                quote! {}
            };

            quote! {
                #static_param_decoding
                #param_decoding
                #data_param_decoding
                #final_byte_decoding
            }
        }
    };

    let constructor = struct_constructor(typename, params, into);

    Ok((decoding, constructor))
}

/// Generate decoding for key-value parameter format.
///
/// In key-value format, parameters are parsed as "key=value" pairs separated
/// by the delimiter. Fields can appear in any order and are matched by name.
fn generate_map_decoding(
    params: &Params,
    props: &impl HasFormatProperties,
    source: &ParamSource,
) -> syn::Result<TokenStream> {
    let fields = &params.fields;
    let field_source_map: HashMap<_, _> = fields
        .iter()
        .map(|field| {
            let source = match &field.mux_index {
                Some(member) => member.clone(),
                None => field.member.clone(),
            };

            (field.member.clone(), source)
        })
        .collect();

    let field_declarations = fields.iter().map(|field| {
        let name = &field.ident();
        let ty = &field.ty;
        if field.is_optional {
            quote! {
                let mut #name: #ty = ::core::option::Option::None;
            }
        } else {
            quote! {
                let mut #name: ::core::option::Option<#ty> = ::core::option::Option::None;
            }
        }
    });

    let setup = quote! {
        #(#field_declarations)*
    };

    let delimiter_lit = &props.delimiter().to_literal();

    let field_arms = fields.iter().map(|field| {
        let name = &field.ident();
        let source = field_source_map
            .get(&field.member)
            .unwrap_or_else(|| panic!("source field for {name}"));
        let source_str = match source {
            syn::Member::Named(ident) => ident.to_string(),
            syn::Member::Unnamed(_) => panic!("unexpected unnamed field source"),
        };
        let source_blit =
            syn::LitByteStr::new(source_str.as_bytes(), proc_macro2::Span::call_site());
        let inner_ty = &field.inner_ty;

        quote! {
            #source_blit => {
                #name = if value.is_empty() {
                    ::core::option::Option::None
                } else {
                    ::core::option::Option::Some(
                        <#inner_ty as ::vtansi::parse::TryFromAnsi>::try_from_ansi(value)?
                    )
                };
            }
        }
    });

    let source_ident = &source.ident;
    let pair_iter = match source.format {
        ParamSourceFormat::Flat => quote! {
            ::vtansi::parse::parse_keyvalue_pairs(#source_ident, #delimiter_lit)
        },
        ParamSourceFormat::Split => quote! {
            ::vtansi::parse::parse_keyvalue_pairs_from_slice(#source_ident)
        },
        ParamSourceFormat::Iter => quote! {
            ::vtansi::parse::parse_keyvalue_pairs_from_iter(#source_ident)
        },
    };

    let field_decoding = quote! {
        for pair_result in #pair_iter {
            let (key, value) = pair_result?;
            match key {
                #(#field_arms)*
                _ => {
                    return ::core::result::Result::Err(
                        ::vtansi::parse::ParseError::InvalidValue(
                            ::std::format!("unknown field: {:?}", key)
                        )
                    );
                }
            }
        }
    };

    // Generate field unwrapping for required fields
    let field_unwrapping = fields.iter().filter_map(|field| {
        let name = &field.ident();
        if field.is_optional {
            // Option fields don't need unwrapping
            None
        } else {
            // Required fields must be present
            let name_str = name.to_string();
            Some(quote! {
                let #name = #name.ok_or_else(|| {
                    ::vtansi::parse::ParseError::InvalidValue(
                        ::std::format!("missing field: {}", #name_str)
                    )
                })?;
            })
        }
    });

    Ok(quote! {
        #setup
        #field_decoding
        #(#field_unwrapping)*
    })
}

/// Generate decoding for vector parameter format.
///
/// In vector format, parameters are parsed as values in declaration order,
/// separated by the delimiter. Required fields must appear before optional
/// fields.
fn generate_vector_decoding(
    params: &Params,
    props: &impl HasFormatProperties,
    source: &ParamSource,
) -> syn::Result<TokenStream> {
    let fields = &params.fields;
    let required_count = params.required_count;
    let delimiter_lit = &props.delimiter().to_literal();
    let total_count = params.total_count;

    // Use a unique iterator name based on the source to avoid shadowing
    // when processing multiple param sources (e.g., static_params vs regular params)
    let source_ident = &source.ident;
    let iter_name = format!("__vtansi_{source_ident}_iter");
    let exhausted_name = format!("__vtansi_{source_ident}_exhausted");
    let prev_name = format!("__vtansi_{source_ident}_prev");

    let param_iterator =
        syn::Ident::new(&iter_name, proc_macro2::Span::mixed_site());
    let params_exhausted =
        syn::Ident::new(&exhausted_name, proc_macro2::Span::mixed_site());
    let prev_params_bytes =
        syn::Ident::new(&prev_name, proc_macro2::Span::mixed_site());

    let mux_setup = if params.has_mux {
        quote! { let mut #prev_params_bytes: [&[u8]; #total_count] = [&[]; #total_count]; }
    } else {
        quote! {}
    };

    let (_iter, setup) = match source.format {
        ParamSourceFormat::Flat => {
            let iter = quote! { #source_ident.split(|&b| b == #delimiter_lit) };
            let setup = quote! {
                let mut #param_iterator = #iter;
                let mut #params_exhausted = false;
                #mux_setup
            };
            (iter, setup)
        }
        ParamSourceFormat::Split => {
            let iter = quote! { #source_ident.into_iter() };
            let setup = quote! {
                let mut #param_iterator = #iter;
                let mut #params_exhausted = false;
                #mux_setup
            };
            (iter, setup)
        }
        ParamSourceFormat::Iter => {
            // For Iter format, the source is already a mutable iterator reference
            // We alias it to param_iterator for consistency with the rest of the code
            let setup = quote! {
                let #param_iterator = #source_ident;
                let mut #params_exhausted = false;
                #mux_setup
            };
            (quote! {}, setup)
        }
    };

    let field_decoding = fields
        .iter()
        .map(|field| {
            let field_type = &field.ty;
            let inner_type = &field.inner_ty;
            let field_name = field.ident();
            let field_idx = field.index;
            let save = if params.has_mux {
                quote! {
                    #prev_params_bytes[#field_idx] = val;
                }
            } else {
                quote! {}
            };

            if field.is_flatten {
                // Flattened field: delegate to try_from_ansi_iter
                quote! {
                    let #field_name = <#field_type as ::vtansi::parse::TryFromAnsiIter>::try_from_ansi_iter(
                        &mut #param_iterator
                    )?;
                }
            } else if let Some(mux_index) = &field.mux_index {
                // This field is sourced from a previous param
                let mux_index = match mux_index {
                    syn::Member::Unnamed(idx) => idx.index,
                    _ => panic!("decoder expected syn::Member::Unnamed, got syn::Member::Named")
                };
                if field.is_optional {
                    quote! {
                        let #field_name: #field_type = if #prev_params_bytes[#field_idx].empty() {
                            ::core::option::Option::None
                        } else {
                            ::core::option::Option::Some(
                                <#inner_type as ::vtansi::parse::TryFromAnsi>::try_from_ansi(
                                    #prev_params_bytes[#mux_index as usize]
                                )?
                            )
                        };
                    }
                } else {
                    quote! {
                        let #field_name: #field_type = <#field_type as ::vtansi::parse::TryFromAnsi>::try_from_ansi(
                            #prev_params_bytes[#mux_index as usize]
                        )?;
                    }
                }
            } else if field.is_optional {
                quote! {
                    let #field_name: #field_type = if #params_exhausted {
                        ::core::option::Option::None
                    } else {
                        match #param_iterator.next() {
                            ::core::option::Option::Some(val) => {
                                if !val.is_empty() {
                                    #save
                                    ::core::option::Option::Some(
                                        <#inner_type as ::vtansi::parse::TryFromAnsi>::try_from_ansi(val)?
                                    )
                                } else {
                                    ::core::option::Option::None
                                }
                            }
                            _ => {
                                #params_exhausted = true;
                                ::core::option::Option::None
                            }
                        }
                    };
                }
            } else {
                quote! {
                    let #field_name = match #param_iterator.next() {
                        ::core::option::Option::Some(val) => {
                            if !val.is_empty() {
                                #save
                                <#field_type as ::vtansi::parse::TryFromAnsi>::try_from_ansi(val)?
                            } else {
                                return ::core::result::Result::Err(
                                    ::vtansi::parse::ParseError::InvalidValue(
                                        ::std::format!(
                                            "parameter data in position {} is empty",
                                            #field_idx,
                                        )
                                    )
                                );
                            }
                        }
                        ::core::option::Option::None => {
                            return ::core::result::Result::Err(
                                ::vtansi::parse::ParseError::InvalidValue(
                                    ::std::format!(
                                        "expected at least {} fields, got {}",
                                        #required_count,
                                        #field_idx,
                                    )
                                )
                            );
                        }
                    };
                }
            }
        });

    let trailer_check = if params.has_flatten {
        // Skip trailer check when flattened fields are present, since they
        // consume a variable number of parameters
        quote! {}
    } else {
        quote! {
            if #param_iterator.next().is_some() {
                return ::core::result::Result::Err(
                    ::vtansi::parse::ParseError::InvalidValue(
                        ::std::format!(
                            "unexpected trailing data, expected at most {} fields",
                            #total_count,
                        )
                    )
                );
            }
        }
    };

    Ok(quote! {
        #setup
        #(#field_decoding)*
        #trailer_check
    })
}

fn struct_constructor(
    typename: &syn::Type,
    params: &StructParamInfo,
    into: Option<&syn::Path>,
) -> TokenStream {
    let idents: Vec<syn::Ident> = params
        .params
        .fields
        .iter()
        .chain(params.data_params.fields.iter())
        .chain(params.final_byte_params.fields.iter())
        .map(|f| f.ident())
        .collect();

    let mut ctor = if idents.is_empty() {
        quote! {
            #typename
        }
    } else if params.named {
        quote! {
            #typename {
                #(#idents),*
            }
        }
    } else {
        quote! {
            #typename (
                #(#idents),*
            )
        }
    };

    // If a custom coercion target is provided, use it
    if let Some(into) = into {
        ctor = quote! {
            #into::try_from(#ctor).map_err(|e| ::vtansi::parse::ParseError::InvalidValue(
                "cannot convert".to_string(),
            ))?
        }
    }

    ctor
}
