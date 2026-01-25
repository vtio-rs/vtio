//! Shared parameter information extraction for structs.
//!
//! This module provides utilities for extracting normalized field
//! information from structs in a way that can be shared between both
//! framed (control sequences) and unframed (plain struct) encoding/decoding.
//!
//! The key insight is that parameter encoding/decoding logic should be
//! identical whether the parameters appear in a control sequence (with
//! intro/final bytes) or as standalone struct values. This module extracts
//! the common parameter information needed by both cases.

use std::collections::HashMap;
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Fields};

use super::metadata::{FieldLocation, StructFormat};
use super::{HasFieldProperties, HasTypeProperties};

/// Information about a single struct field relevant to parameter
/// encoding/decoding.
///
/// This struct contains all the metadata needed to generate encoding or
/// decoding code for a field, regardless of whether it's used in a framed
/// control sequence or unframed struct.
#[derive(Clone, Debug)]
pub struct FieldInfo {
    /// The field's identifier (name) or index (if tuple struct)
    pub member: syn::Member,

    /// The field's type.
    pub ty: syn::Type,

    /// Whether the field is an Option<T> type.
    pub is_optional: bool,

    /// The inner type if this is Option<T>, otherwise the type itself.
    pub inner_ty: syn::Type,

    /// Actual index of the field in the encoded ANSI parameter sequence.
    pub index: usize,

    /// Optional multiplexing target from `#[vtansi(muxwith = field)]`.
    ///
    /// When this is `Some`, the field will be multiplexed with the
    /// referenced field. During encoding, contributes to that field's
    /// parameter via `AnsiMuxEncode` trait. During decoding, parses from
    /// the same parameter as the referenced field.
    pub mux_index: Option<syn::Member>,

    /// Whether this field is flattened via `#[vtansi(flatten)]`.
    ///
    /// When true, the field delegates parameter iterator parsing to its
    /// `try_from_ansi_iter` method instead of consuming a single parameter.
    pub is_flatten: bool,

    /// Whether this field captures from static params position.
    ///
    /// When true, the field parses from position 0 of all_params (including
    /// static params consumed by trie matching), rather than from the
    /// remaining params after static param consumption.
    pub is_static_params: bool,
}

impl FieldInfo {
    pub fn ident(&self) -> syn::Ident {
        match &self.member {
            syn::Member::Named(ident) => ident.clone(),
            syn::Member::Unnamed(idx) => syn::Ident::new(
                &format!("field_{}", idx.index),
                proc_macro2::Span::call_site(),
            ),
        }
    }
}

/// Normalized parameter information extracted from a struct.
///
/// This struct contains all the information needed to generate parameter
/// encoding and decoding code, extracted from a struct's fields and
/// attributes.
#[derive(Clone, Default)]
pub struct StructParamInfo {
    /// Whether the struct is using named fields or is a tuple.
    pub named: bool,

    /// Regular sequence params (i.e encoded as parameter bytes).
    pub params: Params,

    /// Data params (i.e encoded in the data section).
    pub data_params: Params,

    /// Params encoded in the final byte.
    pub final_byte_params: Params,
}

#[derive(Clone, Default)]
pub struct Params {
    /// Information about each field.
    ///
    /// Fields are in declaration order. Skipped fields are excluded from
    /// this list.
    pub fields: Vec<FieldInfo>,

    /// Number of required (non-optional) fields.
    pub required_count: usize,

    /// Total number of encoded parameters (excludes muxed fields since they
    /// share a parameter slot with another field).
    pub total_count: usize,

    /// Whether any fields are multiplexed.
    pub has_mux: bool,

    /// Whether any fields are flattened.
    pub has_flatten: bool,

    /// Whether any fields capture from static params.
    pub has_static_params: bool,
}

impl Params {
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }
}

/// Extract parameter information from a struct.
///
/// Parse the struct's fields and attributes to produce a normalized
/// representation of its parameter structure. This information can be used
/// by both framed and unframed encoding/decoding generators.
///
/// # Errors
///
/// Return an error if:
/// - The input is not a struct with named or unnamed fields
/// - Field attributes cannot be parsed
/// - Required fields appear after optional fields
/// - Type-level attributes are invalid
pub fn extract_struct_param_info(
    ast: &DeriveInput,
    default_format: Option<StructFormat>,
    default_location: FieldLocation,
) -> syn::Result<StructParamInfo> {
    let Data::Struct(data) = &ast.data else {
        return Err(syn::Error::new_spanned(
            ast,
            "extract_struct_param_info only supports structs",
        ));
    };

    let (named, params, data_params, final_byte_params) = match &data.fields {
        Fields::Named(fields) => (
            true,
            extract_fields_info(
                ast,
                fields.named.iter(),
                true,
                default_format.unwrap_or(StructFormat::Map),
                default_location,
                FieldLocation::Params,
            )?,
            extract_fields_info(
                ast,
                fields.named.iter(),
                true,
                default_format.unwrap_or(StructFormat::Map),
                default_location,
                FieldLocation::Data,
            )?,
            extract_fields_info(
                ast,
                fields.named.iter(),
                true,
                StructFormat::Vector,
                default_location,
                FieldLocation::Final,
            )?,
        ),
        Fields::Unnamed(fields) => (
            false,
            extract_fields_info(
                ast,
                fields.unnamed.iter(),
                false,
                default_format.unwrap_or(StructFormat::Vector),
                default_location,
                FieldLocation::Params,
            )?,
            extract_fields_info(
                ast,
                fields.unnamed.iter(),
                false,
                default_format.unwrap_or(StructFormat::Vector),
                default_location,
                FieldLocation::Data,
            )?,
            extract_fields_info(
                ast,
                fields.unnamed.iter(),
                true,
                StructFormat::Vector,
                default_location,
                FieldLocation::Final,
            )?,
        ),
        Fields::Unit => (
            false,
            Params::default(),
            Params::default(),
            Params::default(),
        ),
    };

    Ok(StructParamInfo {
        named,
        params,
        data_params,
        final_byte_params,
    })
}

fn extract_fields_info<'a>(
    ast: &DeriveInput,
    fields: impl Iterator<Item = &'a syn::Field>,
    named: bool,
    default_format: StructFormat,
    default_location: FieldLocation,
    for_location: FieldLocation,
) -> syn::Result<Params> {
    // Check if type has data_fields attribute (only for control sequences)
    let mut field_infos = Vec::new();
    let mut index = 0usize;
    let mut required_count = 0usize;
    let mut first_optional_idx: Option<usize> = None;
    let mut has_mux = false;
    let mut has_flatten = false;
    let mut seen_final_byte_location = false;
    let props = ast.get_format_properties(default_format, named)?;
    let format = props.format;
    let format_is_kv = format == StructFormat::Map;
    let mut field_map: HashMap<syn::Member, (usize, bool)> = HashMap::new();
    let mut mux_fields = Vec::new();

    for (idx, field) in fields.enumerate() {
        let field_props = field.get_field_properties()?;

        // Skip fields with #[vtansi(skip)]
        if field_props.skip.is_some() {
            continue;
        }

        let field_location = field_props.location.unwrap_or(default_location);

        // Filter based on field location
        // StaticParams fields are treated as Params for extraction purposes
        let effective_location =
            if field_location == FieldLocation::StaticParams {
                FieldLocation::Params
            } else {
                field_location
            };
        if for_location != effective_location {
            continue;
        }

        let is_static_params = field_location == FieldLocation::StaticParams;

        if field_location == FieldLocation::Final {
            if seen_final_byte_location {
                return Err(syn::Error::new_spanned(
                    field,
                    "cannot have more than one field encoded as final byte",
                ));
            } else {
                seen_final_byte_location = true;
            }
        }

        let member = match &field.ident {
            Some(ident) => syn::Member::Named(ident.clone()),
            None => syn::Member::Unnamed(syn::Index {
                index: idx as u32,
                span: field.span(),
            }),
        };

        let mux_field = field_props.mux_field;
        let field_has_mux = mux_field.is_some();
        mux_fields.push(mux_field);

        let field_info = FieldInfo {
            member: member.clone(),
            ty: field_props.ty,
            is_optional: field_props.is_optional,
            inner_ty: field_props.inner_ty,
            index,
            mux_index: None, // For now
            is_flatten: field_props.flatten.is_some(),
            is_static_params,
        };

        field_map.insert(member.clone(), (index, field_has_mux));
        field_infos.push(field_info);

        if field_props.is_optional {
            first_optional_idx = Some(index)
        } else if let Some(first_optional_idx) = first_optional_idx
            && !format_is_kv
        {
            return Err(syn::Error::new_spanned(
                field,
                format!(
                    "non-optional field '{member:?}' at position {idx} cannot \
                     appear after optional field at position {first_optional_idx}"
                ),
            ));
        }

        if field_props.flatten.is_some() {
            has_flatten = true;
            // Flattened fields don't contribute to index/count since they
            // consume a variable number of parameters
        } else if !field_has_mux {
            index += 1;

            if !field_props.is_optional {
                required_count += 1
            }
        } else {
            has_mux = true;
        }
    }

    for (field_info, mux_field) in field_infos.iter_mut().zip(mux_fields) {
        if let Some(mux_field) = &mux_field {
            match field_map.get(mux_field) {
                None => {
                    return Err(syn::Error::new(
                        mux_field.span(),
                        "invalid field reference in `muxwith`",
                    ));
                }
                Some((_, true)) => {
                    return Err(syn::Error::new(
                        mux_field.span(),
                        "`muxwith` can only refer to fields that are themselves not annotated as `muxwith`",
                    ));
                }
                Some((index, false)) => {
                    field_info.mux_index =
                        Some(if format == StructFormat::Vector {
                            syn::Member::Unnamed(syn::Index {
                                index: *index as u32,
                                span: proc_macro2::Span::call_site(),
                            })
                        } else {
                            field_info.member.clone()
                        });
                }
            }
        }
    }

    let has_static_params = field_infos.iter().any(|f| f.is_static_params);

    Ok(Params {
        fields: field_infos,
        required_count,
        has_mux,
        has_flatten,
        has_static_params,
        total_count: index,
    })
}
