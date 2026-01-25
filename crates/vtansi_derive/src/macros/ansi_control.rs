//! Implementation of the `AnsiControl` derive macro.
//!
//! This module generates implementations of `AnsiEncode`, `AnsiEvent`, and
//! registry entries for ANSI control sequences (CSI, OSC, DCS, ESC, C0, etc.).
//!
//! The `AnsiControl` derive is specifically designed for control sequences
//! and requires framing attributes like `#[vtansi(csi, finalbyte = 'H')]`.
//!
//! # Examples
//!
//! ```ignore
//! #[derive(AnsiControl)]
//! #[vtansi(csi, finalbyte = 'H')]
//! struct CursorPosition {
//!     row: u16,
//!     col: u16,
//! }
//! ```
//!
//! # Comparison with ToAnsi/FromAnsi
//!
//! - `ToAnsi`/`FromAnsi` - Parameter encoding/decoding only
//! - `AnsiControl` - Complete control sequence framing (uses parameter
//!   encoding internally)

use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{DeriveInput, Fields};

use crate::helpers::generate_doc_imports;

use crate::helpers::StructParamInfo;
use crate::helpers::metadata::ControlDirection;
use crate::helpers::type_props::HasFormatProperties;
use crate::helpers::{
    ControlProperties, WriteOperationBuilder, extract_struct_param_info,
    get_primary_lifetime, metadata::ControlFunctionKind,
    type_props::HasTypeProperties,
};

use crate::macros::param_decoder::{
    ParamSource, ParamSourceFormat, generate_param_decoding,
};
use crate::macros::param_encoder::generate_param_encoding;

/// Generate implementations for the AnsiControl derive.
///
/// This function validates that framing attributes are present and then
/// generates both `TryFromAnsi` and `ToAnsi` implementations using the
/// existing framing logic.
///
/// # Errors
///
/// Return an error if:
/// - No framing attributes are present (csi, osc, dcs, etc.)
/// - The input is not a struct
/// - The framing attributes are invalid
pub fn ansi_control_inner(
    ast: &DeriveInput,
    direction: ControlDirection,
) -> syn::Result<TokenStream> {
    // Validate that this is a struct
    if !matches!(ast.data, syn::Data::Struct(_)) {
        return Err(syn::Error::new_spanned(
            ast,
            "AnsiControl can only be derived for structs, not enums or unions",
        ));
    }

    // Generate doc imports for IDE hover support
    let doc_imports = generate_doc_imports(ast);

    let props = ast.get_control_properties(direction)?;

    // Dispatch based on intro type
    let impl_code = match props.kind {
        ControlFunctionKind::C0 | ControlFunctionKind::Byte => {
            generate_byte_impl(ast, &props)?
        }
        _ => generate_esc_impl(ast, &props)?,
    };

    Ok(quote! {
        #doc_imports
        #impl_code
    })
}

/// Generate AnsiControl trait implementation
///
/// # Errors
///
/// syn::Error
fn generate_control_impl(
    ast: &DeriveInput,
    props: &ControlProperties,
) -> syn::Result<TokenStream> {
    let ty = &ast.ident;
    let (impl_generics, ty_generics, where_clause) =
        ast.generics.split_for_impl();
    let kind = props.kind.as_lib_enum();
    let direction = props.direction.as_lib_enum();
    let (_, trait_lt) = if let Some(lt) = get_primary_lifetime(ast) {
        (quote! { #lt }, quote! { <#lt> })
    } else {
        (quote! {}, quote! { <'_> })
    };

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics ::vtansi::AnsiEvent #trait_lt for #ty #ty_generics #where_clause {
            #[inline]
            fn ansi_control_kind(&self) -> ::core::option::Option<::vtansi::AnsiControlFunctionKind> {
                ::core::option::Option::Some(#kind)
            }

            #[inline]
            fn ansi_direction(&self) -> ::vtansi::AnsiControlDirection {
                #direction
            }

            ::vtansi::impl_ansi_event_encode!();
            ::vtansi::impl_ansi_event_terse_fmt!();
        }
    })
}

/// Generate `TerseDisplay` trait implementation.
///
/// This generates an implementation that delegates to `Debug` for rendering
/// the innards of the event.
///
/// # Errors
///
/// syn::Error
fn generate_terse_display_impl(ast: &DeriveInput) -> syn::Result<TokenStream> {
    let ty = &ast.ident;
    let (impl_generics, ty_generics, where_clause) =
        ast.generics.split_for_impl();

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics ::vtansi::TerseDisplay for #ty #ty_generics #where_clause {
            #[inline]
            fn terse_fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                ::std::fmt::Debug::fmt(self, f)
            }
        }
    })
}

/// Generate better_any::Tid implementation
///
/// # Errors
///
/// syn::Error
fn generate_tid_impl(ast: &DeriveInput) -> syn::Result<TokenStream> {
    let ty = &ast.ident;

    // For types with a lifetime parameter, use the full impl syntax
    // For types without lifetimes, use the simple form
    let tid_impl = if let Some(lt) = get_primary_lifetime(ast) {
        quote! {
            #[automatically_derived]
            ::vtansi::__private::better_any::tid! { #ty<#lt> }
        }
    } else {
        quote! {
            #[automatically_derived]
            ::vtansi::__private::better_any::tid! { #ty }
        }
    };

    Ok(tid_impl)
}

/// Validate SS3 sequence constraints.
///
/// SS3 sequences must either:
/// 1. Define static data (no fields), OR
/// 2. Have exactly one field (which defaults to data location)
///
/// # Errors
///
/// Returns an error if the SS3 sequence has multiple fields or has fields in
/// non-data locations.
fn validate_osc_sequence(ast: &DeriveInput) -> syn::Result<()> {
    use crate::helpers::HasFieldProperties;
    use crate::helpers::metadata::FieldLocation;
    use syn::Field;

    let syn::Data::Struct(data) = &ast.data else {
        return Ok(());
    };

    let fields: Box<dyn Iterator<Item = &Field>> = match &data.fields {
        Fields::Named(f) => Box::new(f.named.iter()),
        Fields::Unnamed(f) => Box::new(f.unnamed.iter()),
        Fields::Unit => return Ok(()),
    };

    for field in fields {
        let field_props = field.get_field_properties()?;

        // Check if locate was explicitly specified
        if let Some(location) = field_props.location {
            match location {
                FieldLocation::Data => {
                    // Pointless but allowed - data is the default for OSC sequences.
                    // Users should remove the redundant attribute.
                }
                FieldLocation::Params => {
                    return Err(syn::Error::new_spanned(
                        field,
                        "OSC sequences do not support #[vtansi(locate = \"params\")].\n\
                         \n\
                         All fields in OSC sequences must be in the data location.\n\
                         Remove the locate attribute.",
                    ));
                }
                FieldLocation::Final => {
                    return Err(syn::Error::new_spanned(
                        field,
                        "OSC sequences do not support #[vtansi(locate = \"final\")].\n\
                         \n\
                         OSC sequences have no final byte - they are terminated by ST.\n\
                         Remove the locate attribute.",
                    ));
                }
                FieldLocation::StaticParams => {
                    return Err(syn::Error::new_spanned(
                        field,
                        "OSC sequences do not support #[vtansi(locate = \"static_params\")].\n\
                         \n\
                         OSC sequences do not have static params in the trie.\n\
                         Remove the locate attribute.",
                    ));
                }
            }
        }
    }

    Ok(())
}

fn validate_ss3_sequence(
    ast: &DeriveInput,
    props: &ControlProperties,
    params: &StructParamInfo,
) -> syn::Result<()> {
    // Count total fields across all locations
    let total_fields = params.params.fields.len()
        + params.data_params.fields.len()
        + params.final_byte_params.fields.len();

    // Check if there's static data defined
    let has_static_data = !props.data.is_empty();

    if has_static_data {
        // If static data is defined, there should be no fields
        if total_fields > 0 {
            return Err(syn::Error::new_spanned(
                ast,
                "SS3 sequences with static data cannot have fields.\n\
                 \n\
                 Either:\n\
                 - Remove the `data` attribute and add a single field, OR\n\
                 - Remove all fields to use static data only",
            ));
        }
    } else {
        // No static data - must have exactly one field in data location
        if total_fields == 0 {
            return Err(syn::Error::new_spanned(
                ast,
                "SS3 sequences must either define static `data` or have exactly one field.\n\
                 \n\
                 Add one of:\n\
                 - #[vtansi(ss3, finalbyte = 'A', data = \"foo\")] for static data, OR\n\
                 - A single field in your struct (which defaults to data location)",
            ));
        }

        if total_fields > 1 {
            return Err(syn::Error::new_spanned(
                ast,
                "SS3 sequences can only have a single field.\n\
                 \n\
                 SS3 sequences accept exactly one field which is placed in the data section.",
            ));
        }

        // Ensure the single field is in the data location (not params or final)
        if !params.params.fields.is_empty() {
            return Err(syn::Error::new_spanned(
                ast,
                "SS3 sequence fields must be in the data location, not params.\n\
                 \n\
                 Use #[vtansi(locate = \"data\")] on the field, or set \n\
                 #[vtansi(ss3, finalbyte = 'A', locate_all = \"data\")] on the struct.",
            ));
        }

        if !params.final_byte_params.fields.is_empty() {
            return Err(syn::Error::new_spanned(
                ast,
                "SS3 sequence fields must be in the data location, not final byte.\n\
                 \n\
                 Use #[vtansi(locate = \"data\")] on the field, or set \n\
                 #[vtansi(ss3, finalbyte = 'A', locate_all = \"data\")] on the struct.",
            ));
        }
    }

    Ok(())
}

/// Generate implementation for C0/Byte control sequences.
///
/// C0 and Byte sequences are represented as a single byte code and
/// implement `StaticAnsiEncode` with a constant byte slice.
fn generate_byte_impl(
    ast: &DeriveInput,
    props: &ControlProperties,
) -> syn::Result<TokenStream> {
    let struct_name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) =
        ast.generics.split_for_impl();

    if let Some(code) = props.code {
        let default_format =
            Some(crate::helpers::metadata::StructFormat::Vector);
        let params = extract_struct_param_info(
            ast,
            default_format,
            props.field_location,
        )?;
        let registry = generate_registry_entries(ast, props, &params)?;
        let control_impl = generate_control_impl(ast, props)?;
        let tid_impl = generate_tid_impl(ast)?;
        let terse_display_impl = generate_terse_display_impl(ast)?;
        let encode_impl = quote! {
            impl #impl_generics ::vtansi::StaticAnsiEncode for #struct_name #ty_generics #where_clause {
                const BYTES: &'static [u8] = ::std::slice::from_ref(&#code);
            }
        };

        Ok(quote! {
            #registry
            #control_impl
            #tid_impl
            #terse_display_impl
            #encode_impl
        })
    } else {
        Err(crate::helpers::error_required_attr(
            ast.span(),
            "code",
            "0x0E",
        ))
    }
}

/// Generate implementation for escape-based control sequences.
///
/// This handles CSI, OSC, DCS, ESC, ESCST, and SS3 sequences, generating
/// `AnsiEncode`, `AnsiEvent`, and registry entries.
fn generate_esc_impl(
    ast: &DeriveInput,
    props: &ControlProperties,
) -> syn::Result<TokenStream> {
    let default_format = Some(crate::helpers::metadata::StructFormat::Vector);
    let params =
        extract_struct_param_info(ast, default_format, props.field_location)?;

    // Validate SS3 sequences
    if props.kind == ControlFunctionKind::Ss3 {
        validate_ss3_sequence(ast, props, &params)?;
    }

    // Validate OSC sequences
    if props.kind == ControlFunctionKind::Osc {
        validate_osc_sequence(ast)?;
    }

    let control_impl = generate_control_impl(ast, props)?;
    let tid_impl = generate_tid_impl(ast)?;
    let terse_display_impl = generate_terse_display_impl(ast)?;
    let registry = generate_registry_entries(ast, props, &params)?;
    let encode_impl = generate_esc_encode_impl(ast, props, &params)?;
    // let decode_impl = generate_esc_decode_impl(ast, props, &params)?;

    Ok(quote! {
        #registry
        #control_impl
        #tid_impl
        #terse_display_impl
        #encode_impl
        // #decode_impl
    })
}

#[allow(dead_code)]
fn generate_esc_decode_impl(
    ast: &DeriveInput,
    props: &ControlProperties,
    params: &StructParamInfo,
) -> syn::Result<TokenStream> {
    let params_ident =
        syn::Ident::new("__vtansi_params", proc_macro2::Span::mixed_site());
    let stype: syn::Type = syn::parse_quote!(Self);
    let (params_decoding, constructor) = generate_param_decoding(
        &stype,
        params,
        &props,
        &ParamSource::new(&params_ident, ParamSourceFormat::Flat),
        None, // static_params_source - ESC sequences don't have static params
        None,
        None,
        props.into.as_ref(),
    )?;
    let name = &ast.ident;
    let generics = &ast.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let (lt, trait_lt) = if let Some(lt) = get_primary_lifetime(ast) {
        (quote! { #lt }, quote! { <#lt> })
    } else {
        (quote! {}, quote! { <'_> })
    };

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics ::vtansi::parse::TryFromAnsi #trait_lt for #name #ty_generics #where_clause {
            #[inline]
            fn try_from_ansi(#params_ident: &#lt [u8]) -> ::core::result::Result<#name #ty_generics, ::vtansi::parse::ParseError> {
                ::core::result::Result::Ok({
                    #params_decoding
                    #constructor
                })
            }
        }
    })
}

fn control_is_const(
    props: &ControlProperties,
    params: &StructParamInfo,
) -> bool {
    params.params.fields.is_empty()
        && params.data_params.fields.is_empty()
        && params.final_byte_params.fields.is_empty()
        && props.final_bytes.len() < 2
}

fn generate_esc_encode_impl(
    ast: &DeriveInput,
    props: &ControlProperties,
    params: &StructParamInfo,
) -> syn::Result<TokenStream> {
    if control_is_const(props, params) {
        generate_esc_encode_const_impl(ast, props, params)
    } else {
        generate_esc_encode_dynamic_impl(ast, props, params)
    }
}

fn generate_esc_encode_const_impl(
    ast: &DeriveInput,
    props: &ControlProperties,
    _params: &StructParamInfo,
) -> syn::Result<TokenStream> {
    let prefix = props.get_static_prefix();
    let suffix = if let Some(suffix) = props.get_static_suffix() {
        suffix
    } else {
        panic!("unexpected dynamic final byte")
    };

    let name = &ast.ident;
    let generics = &ast.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let bytes = [
        prefix.as_slice(),
        suffix.as_slice(),
        props.get_static_data().as_slice(),
        props.get_terminator().as_slice(),
    ]
    .concat();

    let bytes_lit =
        syn::LitByteStr::new(&bytes, proc_macro2::Span::mixed_site());

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics ::vtansi::encode::StaticAnsiEncode for #name #ty_generics #where_clause {
            const BYTES: &'static [u8] = #bytes_lit;
        }
    })
}

fn generate_esc_encode_dynamic_impl(
    ast: &DeriveInput,
    props: &ControlProperties,
    params: &StructParamInfo,
) -> syn::Result<TokenStream> {
    let sink =
        syn::Ident::new("__vtansi_sink", proc_macro2::Span::mixed_site());
    let counter =
        syn::Ident::new("__vtansi_ctr", proc_macro2::Span::mixed_site());

    let name = &ast.ident;

    let mut prefix_writes = WriteOperationBuilder::new(&sink, &counter);
    let prefix_len = props.write_static_prefix(&mut prefix_writes);
    let prefix_writes = prefix_writes.build();

    let mut suffix_writes = WriteOperationBuilder::new(&sink, &counter);
    let suffix_len = props.write_static_suffix(&mut suffix_writes);
    let suffix_writes = suffix_writes.build();

    let mut data_writes = WriteOperationBuilder::new(&sink, &counter);
    let mut data_prefix_len = props.write_static_data(&mut data_writes);
    let data_writes = data_writes.build();

    let mut terminator_writes = WriteOperationBuilder::new(&sink, &counter);
    let terminator_len = props.write_terminator(&mut terminator_writes);
    let terminator_writes = terminator_writes.build();

    let mut generics = ast.generics.clone();
    let where_clause = generics.make_where_clause();

    if props.final_bytes.len() > 1 {
        let (_, ty_generics, _) = ast.generics.split_for_impl();
        where_clause.predicates.push(syn::parse_quote!(
            #name #ty_generics: ::vtansi::encode::AnsiFinalByte
        ));
    }

    let mut len_sum_terms: Vec<proc_macro2::TokenStream> =
        vec![quote! { ::core::option::Option::Some(#prefix_len) }];

    for field in &params.params.fields {
        if field.mux_index.is_none() {
            let ty = &field.inner_ty;
            len_sum_terms.push(
                quote! { <#ty as ::vtansi::encode::AnsiEncode>::ENCODED_LEN },
            )
        }
    }

    for field in &params.data_params.fields {
        if field.mux_index.is_none() {
            let ty = &field.inner_ty;
            len_sum_terms.push(
                quote! { <#ty as ::vtansi::encode::AnsiEncode>::ENCODED_LEN },
            )
        }
    }

    for field in &params.final_byte_params.fields {
        if field.mux_index.is_none() {
            let ty = &field.inner_ty;
            len_sum_terms.push(
                quote! { <#ty as ::vtansi::encode::AnsiEncode>::ENCODED_LEN },
            )
        }
    }

    if suffix_len > 0 {
        len_sum_terms
            .push(quote! { ::core::option::Option::Some(#suffix_len) });
    }

    if terminator_len > 0 {
        len_sum_terms
            .push(quote! { ::core::option::Option::Some(#terminator_len) });
    }

    if data_prefix_len > 0
        && !params.data_params.is_empty()
        && props.data_delimiter.is_some()
    {
        data_prefix_len += 1
    }

    if data_prefix_len > 0 {
        len_sum_terms
            .push(quote! { ::core::option::Option::Some(#data_prefix_len) });
    };

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Generate pattern matching code to sum Option<usize> values.
    // If all are Some, return Some(sum). If any is None, return None.
    let len_sum_vars: Vec<_> = (0..len_sum_terms.len())
        .map(|i| {
            syn::Ident::new(
                &format!("__len_{i}"),
                proc_macro2::Span::mixed_site(),
            )
        })
        .collect();

    let len_expr = quote! {
        match ( #( #len_sum_terms ),* ) {
            ( #( ::core::option::Option::Some(#len_sum_vars) ),* ) => {
                ::core::option::Option::Some( 0 #( + #len_sum_vars )* )
            }
            _ => ::core::option::Option::None,
        }
    };

    let param_writes = generate_param_encoding(
        &params.params,
        &props.get_param_encoding(),
        &sink,
        &counter,
    )?;

    let data_param_writes = generate_param_encoding(
        &params.data_params,
        &props.get_data_param_encoding(),
        &sink,
        &counter,
    )?;

    let final_byte_param_writes = generate_param_encoding(
        &params.final_byte_params,
        &props.get_final_byte_param_encoding(),
        &sink,
        &counter,
    )?;

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics ::vtansi::encode::AnsiEncode for #name #ty_generics #where_clause {
            const ENCODED_LEN: ::core::option::Option<usize> = #len_expr;

            #[inline]
            fn encode_ansi_into<W: ::std::io::Write + ?::std::marker::Sized>(
                &self,
                #sink: &mut W,
            ) -> Result<usize, ::vtansi::EncodeError> {
                let mut #counter = 0usize;
                #(#prefix_writes)*
                #param_writes
                #(#suffix_writes)*
                #(#data_writes)*
                #data_param_writes
                #final_byte_param_writes
                #(#terminator_writes)*
                Ok(#counter)
            }
        }
    })
}

/// Generate registry entry (or entries) and handler for a control function.
///
/// Create the distributed slice entry that registers this control function
/// with the parser, along with a handler function that will be called when
/// the sequence is recognized.
///
/// This generates ONE handler function and multiple registry entries for each
/// combination of (params_alternative, final_byte, param_marker, has_data_params).
/// All registry entries point to the same handler.
pub fn generate_registry_entries(
    ast: &DeriveInput,
    props: &ControlProperties,
    params: &StructParamInfo,
) -> syn::Result<TokenStream> {
    use crate::helpers::metadata::AnsiStrings;

    // Skip registry registration for alias types
    if props.alias_of.is_some() {
        return Ok(quote! {});
    }

    let struct_name = &ast.ident;
    let struct_name_str = struct_name.to_string();

    let handler_name_str =
        format!("{}_handler", struct_name_str.to_lowercase());
    let handler_name_ident =
        syn::Ident::new(&handler_name_str, struct_name.span());
    let cb = syn::Ident::new("__vtansi_cb", proc_macro2::Span::mixed_site());
    let event_data =
        syn::Ident::new("__vtansi_event_data", proc_macro2::Span::mixed_site());
    let param_source =
        syn::Ident::new("__vtansi_params", proc_macro2::Span::mixed_site());
    let static_param_source = syn::Ident::new(
        "__vtansi_static_params",
        proc_macro2::Span::mixed_site(),
    );
    let data_param_source =
        syn::Ident::new("__vtansi_data", proc_macro2::Span::mixed_site());
    let final_byte_source =
        syn::Ident::new("__vtansi_finalbyte", proc_macro2::Span::mixed_site());
    let event =
        syn::Ident::new("__vtansi_event", proc_macro2::Span::mixed_site());
    let stype: syn::Type = syn::parse_quote!(#struct_name);

    // Only create static_params_source if there are fields that need it
    let static_params_source = if params.params.has_static_params {
        Some(ParamSource::new(
            &static_param_source,
            ParamSourceFormat::Split,
        ))
    } else {
        None
    };

    let (param_decoding, constructor) = generate_param_decoding(
        &stype,
        params,
        props,
        &ParamSource::new(&param_source, ParamSourceFormat::Split),
        static_params_source.as_ref(),
        Some(&ParamSource::new(
            &data_param_source,
            ParamSourceFormat::Flat,
        )),
        Some(&ParamSource::new(
            &final_byte_source,
            ParamSourceFormat::Flat,
        )),
        props.into.as_ref(),
    )?;
    let kind = props.kind.as_lib_enum();
    // Use the first params alternative for the prefix (used in registry entry metadata)
    let prefix = props.get_static_prefix();
    let intro_bytes = props.kind.introducer();
    let prefix_bytes = syn::LitByteStr::new(
        &prefix[intro_bytes.len()..],
        proc_macro2::Span::mixed_site(),
    );
    let direction = props.direction.as_ref().to_ascii_uppercase();
    let registry = format!("ANSI_CONTROL_{direction}_FUNCTION_REGISTRY");
    let registry_list = syn::Ident::new(&registry, struct_name.span());

    // Determine if this sequence has required params (for trie key disambiguation)
    // For CSI sequences with all-optional params, we need to emit two entries:
    // one for the no-params case and one for the has-params case.
    // Flatten fields consume params but don't count towards required_count/total_count,
    // so we need to check has_flatten separately.
    let has_required_params = params.params.required_count > 0
        || !props.params.is_empty()
        || params.params.has_flatten;
    let has_all_optional_params = params.params.required_count == 0
        && (params.params.total_count > 0 || params.params.has_flatten)
        && props.params.is_empty()
        && props.kind == ControlFunctionKind::Csi;

    // For disambiguated sequences, validate constraints and compute param marker
    // The param marker encoding is:
    // - 0x00 = no params
    // - 0x01 = has params (normal case)
    // - 0x02 + N = exactly N params (disambiguated sequences)
    if props.disambiguate {
        // Disambiguated sequences must not have optional params
        if params.params.required_count != params.params.total_count {
            return Err(syn::Error::new_spanned(
                ast,
                "disambiguate requires all parameters to be required (no optional params)",
            ));
        }
        // Must be a CSI sequence
        if props.kind != ControlFunctionKind::Csi {
            return Err(syn::Error::new_spanned(
                ast,
                "disambiguate is only supported for CSI sequences",
            ));
        }
    }

    // Compute the param marker for this sequence
    let param_marker: u8 = if props.disambiguate {
        // Disambiguated: use 2 + total_params
        #[allow(clippy::cast_possible_truncation)]
        let marker = 2u8.saturating_add(params.params.total_count as u8);
        marker
    } else if has_required_params {
        1 // Normal: has params
    } else {
        0 // Normal: no params
    };

    // For OSC sequences with data and all-optional data params, we need to emit
    // two entries: one without the trailing delimiter (for no data params) and
    // one with the trailing delimiter (for when data params are provided).
    let has_all_optional_data_params = params.data_params.required_count == 0
        && params.data_params.total_count > 0
        && props.kind == ControlFunctionKind::Osc
        && !props.data.is_empty();

    // Generate static params extraction if needed
    let static_params_extraction = if params.params.has_static_params {
        quote! {
            let mut #static_param_source = #event_data.iter_static_params().unwrap_or_default();
        }
    } else {
        quote! {}
    };

    // Generate ONE handler function that all registry entries will share.
    let handler_fn = quote! {
        #[automatically_derived]
        #[doc(hidden)]
        fn #handler_name_ident<'a>(
            #event_data: &'a ::vtansi::registry::AnsiEventData<'a>,
            #cb: &mut ::vtansi::registry::AnsiEmitFn,
        ) -> ::core::result::Result<(), ::vtansi::parse::ParseError> {
            let mut #param_source = #event_data.iter_params().unwrap_or_default();
            #static_params_extraction
            let #data_param_source = #event_data.get_data().unwrap_or_default();
            let #final_byte_source = #event_data.get_finalbyte().unwrap_or_default();
            #param_decoding
            let #event = #constructor;
            #cb(&#event);
            Ok(())
        }
    };

    // Closure to emit a registry entry (without handler - all share the same handler)
    let emit_entry = |suffix: usize,
                      final_byte: Option<u8>,
                      param_marker: u8,
                      has_data_params: bool,
                      static_params: &AnsiStrings|
     -> proc_macro2::TokenStream {
        // Generate trie key with the specific final byte, param marker, and params alternative
        let key_bytes = syn::LitByteStr::new(
            &props.get_key_with_params(
                final_byte,
                param_marker,
                has_data_params,
                static_params,
            ),
            proc_macro2::Span::mixed_site(),
        );

        let registry_name = format!(
            "__{}_REGISTRY_ENTRY_{}",
            struct_name_str.to_uppercase(),
            suffix,
        );

        let registry_name = syn::Ident::new(&registry_name, struct_name.span());
        let final_byte_token = if let Some(final_byte) = final_byte {
            quote! { ::core::option::Option::Some(#final_byte) }
        } else {
            quote! { ::core::option::Option::None }
        };

        quote! {
            #[doc(hidden)]
            #[::vtansi::__private::linkme::distributed_slice(::vtansi::registry::#registry_list)]
            static #registry_name: ::vtansi::registry::AnsiControlFunctionMatchEntry =
                ::vtansi::registry::AnsiControlFunctionMatchEntry {
                    name: #struct_name_str,
                    key: #key_bytes,
                    kind: #kind,
                    prefix: #prefix_bytes,
                    final_byte: #final_byte_token,
                    handler: #handler_name_ident,
                };
        }
    };

    // Build list of (final_byte, param_marker, has_data_params) tuples for base entries
    // These will be combined with params alternatives to produce the full entry list
    let default_has_data_params = !params.data_params.is_empty();
    let base_specs: Vec<(Option<u8>, u8, bool)> = match props.final_bytes.len()
    {
        0 => {
            if props.disambiguate {
                vec![(None, param_marker, default_has_data_params)]
            } else if has_all_optional_params {
                // Emit two entries: one with param_marker=0, one with param_marker=1
                vec![
                    (None, 0, default_has_data_params),
                    (None, 1, default_has_data_params),
                ]
            } else if has_all_optional_data_params {
                // Emit two entries: one without trailing delimiter, one with
                vec![(None, param_marker, false), (None, param_marker, true)]
            } else {
                vec![(None, param_marker, default_has_data_params)]
            }
        }
        1 => {
            let fb = Some(*props.final_bytes[0]);
            if props.disambiguate {
                vec![(fb, param_marker, default_has_data_params)]
            } else if has_all_optional_params {
                vec![
                    (fb, 0, default_has_data_params),
                    (fb, 1, default_has_data_params),
                ]
            } else if has_all_optional_data_params {
                vec![(fb, param_marker, false), (fb, param_marker, true)]
            } else {
                vec![(fb, param_marker, default_has_data_params)]
            }
        }
        _ => {
            if props.disambiguate {
                props
                    .final_bytes
                    .iter()
                    .map(|b| (Some(**b), param_marker, default_has_data_params))
                    .collect()
            } else if has_all_optional_params {
                // For multiple final bytes with all-optional params,
                // emit two entries per final byte
                props
                    .final_bytes
                    .iter()
                    .flat_map(|b| {
                        let fb = Some(**b);
                        vec![
                            (fb, 0, default_has_data_params),
                            (fb, 1, default_has_data_params),
                        ]
                    })
                    .collect()
            } else if has_all_optional_data_params {
                props
                    .final_bytes
                    .iter()
                    .flat_map(|b| {
                        let fb = Some(**b);
                        vec![
                            (fb, param_marker, false),
                            (fb, param_marker, true),
                        ]
                    })
                    .collect()
            } else {
                props
                    .final_bytes
                    .iter()
                    .map(|b| (Some(**b), param_marker, default_has_data_params))
                    .collect()
            }
        }
    };

    // Get params alternatives, defaulting to a single empty alternative if none specified
    let params_alts: Vec<&AnsiStrings> = if props.params.is_empty() {
        // Create a reference to an empty AnsiStrings for the single "no static params" case
        vec![props.params.first()]
    } else {
        props.params.iter().collect()
    };

    // Generate registry entries for each combination of (params_alt, base_spec)
    // Build the full list of combinations first, then emit entries with proper indices
    let mut entries: Vec<proc_macro2::TokenStream> = Vec::new();
    for (params_idx, static_params) in params_alts.iter().enumerate() {
        for (spec_idx, (fb, marker, has_data_params)) in
            base_specs.iter().enumerate()
        {
            let entry_idx = params_idx * base_specs.len() + spec_idx;
            entries.push(emit_entry(
                entry_idx,
                *fb,
                *marker,
                *has_data_params,
                static_params,
            ));
        }
    }

    Ok(quote! {
        #handler_fn
        #(#entries)*
    })
}
