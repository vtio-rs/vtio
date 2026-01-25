//! Type-level properties and metadata extraction.
//!
//! This module provides utilities for extracting metadata from type-level
//! attributes on enums and structs, such as `#[repr(...)]` for enums or
//! `#[vtansi(format = "...")]` for structs. The extracted properties are
//! used to determine how to generate trait implementations.

use quote::quote;
use syn::{Data, DeriveInput, Fields, Ident};

use crate::helpers::WriteOperationBuilder;
use crate::helpers::metadata::{AnsiChars, ControlDirection, FieldLocation};

use super::metadata::{
    AnsiChar, AnsiString, AnsiStrings, AnsiStringsAlts, ControlFunctionKind,
    ControlKeyword, DeriveInputExt, StructFormat, TypeMeta,
};
use super::occurrence_error;
use super::repr_type::extract_repr_type;
use super::type_analysis::is_vec_type;

pub struct ParamEncoding {
    pub format: StructFormat,
    pub delimiter: Option<AnsiChar>,
    pub offset: usize,
}

/// Trait for extracting vtansi type properties from AST nodes.
///
/// This trait provides a uniform interface for extracting type-level
/// metadata from derive input AST nodes. Implementations scan the type's
/// attributes and return a structured representation of the vtansi-relevant
/// properties.
pub trait HasTypeProperties {
    /// Extract vtansi properties from this type.
    ///
    /// This method processes the type's attributes and returns a
    /// `TypeProperties` struct containing all relevant metadata for code
    /// generation.
    ///
    /// # Errors
    ///
    /// Return an error if the attributes cannot be parsed or contain
    /// invalid values.
    fn get_type_properties(&self) -> syn::Result<ValueProperties>;

    fn get_format_properties(
        &self,
        default_format: StructFormat,
        named: bool,
    ) -> syn::Result<FormatProperties>;

    fn get_control_properties(
        &self,
        direction: ControlDirection,
    ) -> syn::Result<ControlProperties>;
}

pub trait HasFormatProperties {
    fn format(&self) -> StructFormat;
    fn delimiter(&self) -> AnsiChar;
    fn offset(&self) -> usize;

    #[must_use]
    fn get_param_encoding(&self) -> ParamEncoding {
        ParamEncoding {
            format: self.format(),
            delimiter: Some(self.delimiter()),
            offset: self.offset(),
        }
    }
}

impl<T: ?Sized> HasFormatProperties for &T
where
    T: HasFormatProperties,
{
    #[inline]
    fn delimiter(&self) -> AnsiChar {
        <T as HasFormatProperties>::delimiter(*self)
    }

    #[inline]
    fn format(&self) -> StructFormat {
        <T as HasFormatProperties>::format(*self)
    }

    #[inline]
    fn offset(&self) -> usize {
        <T as HasFormatProperties>::offset(*self)
    }
}

pub struct FormatProperties {
    /// The struct encoding format from `#[vtansi(format = "...")]`.
    ///
    /// For structs, this determines whether fields are encoded as
    /// `key=value` pairs or as values only. Defaults to `Map`.
    pub format: StructFormat,

    /// The field delimiter from `#[vtansi(delimiter = "...")]`.
    ///
    /// For structs, this determines what separator is used between fields.
    /// Defaults to `";"`.
    pub delimiter: AnsiChar,
}

impl HasFormatProperties for FormatProperties {
    fn delimiter(&self) -> AnsiChar {
        self.delimiter.clone()
    }

    fn format(&self) -> StructFormat {
        self.format
    }

    fn offset(&self) -> usize {
        0
    }
}

/// Properties extracted from a type's attributes.
///
/// This struct holds metadata parsed from attributes like `#[repr(...)]` on
/// enum declarations or `#[vtansi(...)]` on struct declarations. It is used
/// during code generation to determine the appropriate implementation
/// strategy.
#[derive(Clone)]
pub struct ValueProperties {
    /// The primitive representation type from `#[repr(...)]`, if present.
    ///
    /// When an enum has a `#[repr(u8)]` or similar attribute, this field
    /// contains the primitive type identifier. This determines whether the
    /// enum uses integer-based or string-based conversion.
    pub repr_type: Option<Ident>,

    /// The struct encoding format from `#[vtansi(format = "...")]`.
    ///
    /// For structs, this determines whether fields are encoded as
    /// `key=value` pairs or as values only. Defaults to `Map`.
    pub format: StructFormat,

    /// The field delimiter from `#[vtansi(delimiter = "...")]`.
    ///
    /// For structs, this determines what separator is used between fields.
    /// Defaults to `";"`.
    pub delimiter: AnsiChar,

    /// Whether the struct is transparent from `#[vtansi(transparent)]`.
    ///
    /// When true, the struct should serialize as its single field directly.
    pub transparent: bool,

    /// The constant encoded length from `#[vtansi(encoded_len = N)]`.
    ///
    /// When present, this value will be used for the `ENCODED_LEN`
    /// associated constant in the generated `AnsiEncode` impl.
    /// Only valid for enums without `#[repr(...)]` attributes.
    pub encoded_len: Option<syn::LitInt>,

    /// Optional target type to convert into when parsing.
    pub into: Option<syn::Path>,
}

impl HasFormatProperties for ValueProperties {
    fn format(&self) -> StructFormat {
        self.format
    }

    fn delimiter(&self) -> AnsiChar {
        self.delimiter.clone()
    }

    fn offset(&self) -> usize {
        0
    }
}

/// Properties extracted from a type's attributes.
///
/// This struct holds metadata parsed from attributes like `#[repr(...)]` on
/// enum declarations or `#[vtansi(...)]` on struct declarations. It is used
/// during code generation to determine the appropriate implementation
/// strategy.
#[derive(Clone)]
pub struct ControlProperties {
    /// The struct encoding format from `#[vtansi(format = "...")]`.
    ///
    /// For structs, this determines whether fields are encoded as
    /// `key=value` pairs or as values only. Defaults to `Map`.
    pub format: StructFormat,

    /// Whhere all fields are located by default.
    pub field_location: FieldLocation,

    /// The field delimiter from `#[vtansi(delimiter = "...")]`.
    ///
    /// For structs, this determines what separator is used between fields.
    /// Defaults to `";"`.
    pub delimiter: AnsiChar,

    /// The control function direction (host->terminal or terminal->host).
    pub direction: ControlDirection,

    /// The escape sequence introducer type (CSI, ESC, OSC, etc).
    pub kind: ControlFunctionKind,

    /// Optional private marker byte (one of '<', '=', '>', `?`).
    pub private: Option<AnsiChar>,

    /// Alternative parameter byte sequences (const params).
    /// Each alternative is a list of parameters.
    /// Supports syntax like `params = ['12'] | ['13']`.
    pub params: AnsiStringsAlts,

    /// Intermediate byte sequence.
    pub intermediate: Option<AnsiString>,

    /// Final byte(s) that terminate the sequence.
    pub final_bytes: AnsiChars,

    /// Optional data string (for DCS sequences) that appears after the
    /// final byte but before the string terminator.
    pub data: AnsiString,

    /// Optional numeric parameter for OSC sequences (Ps in ESC ] Ps; Pt
    /// ST).
    pub number: Option<AnsiString>,

    /// Custom separator between static data and first data parameter
    /// (default: ";").
    pub data_delimiter: Option<AnsiChar>,

    /// C0 control code (for C0 sequences only).
    pub code: Option<u8>,

    /// Optional custom type path from `#[vtansi(into = path)]`.
    pub into: Option<syn::Path>,

    /// Optional alias target type from `#[vtansi(alias_of = PrimaryType)]`.
    ///
    /// When set, this type is an alias for another type and should not
    /// register in the parser trie. It will still encode to the same
    /// byte sequence.
    pub alias_of: Option<syn::Path>,

    /// Whether this sequence uses exact param count for disambiguation.
    ///
    /// When true, the sequence uses `2 + total_params` as the key byte
    /// instead of the boolean `has_params`. This allows sequences with
    /// the same final byte but different param counts to coexist.
    ///
    /// The sequence must not have optional parameters when this is enabled.
    pub disambiguate: bool,
}

impl HasFormatProperties for ControlProperties {
    fn format(&self) -> StructFormat {
        self.format
    }

    fn delimiter(&self) -> AnsiChar {
        self.delimiter.clone()
    }

    fn offset(&self) -> usize {
        // Use the first alternative's length as the offset
        self.params.first().len()
    }
}

impl ControlProperties {
    #[must_use]
    pub fn get_final_byte_param_encoding(&self) -> ParamEncoding {
        ParamEncoding {
            format: StructFormat::Vector,
            delimiter: None,
            offset: 0,
        }
    }

    #[must_use]
    pub fn get_data_param_encoding(&self) -> ParamEncoding {
        ParamEncoding {
            format: self.format,
            delimiter: self.data_delimiter.clone(),
            offset: if !self.data.is_empty() { 1 } else { 0 },
        }
    }

    /// Get static prefix using the first params alternative (for encoding).
    #[must_use]
    pub fn get_static_prefix(&self) -> Vec<u8> {
        self.get_static_prefix_with_params(self.params.first())
    }

    /// Get static prefix with a specific params set.
    #[must_use]
    pub fn get_static_prefix_with_params(
        &self,
        params: &AnsiStrings,
    ) -> Vec<u8> {
        let mut buf: Vec<u8> = Vec::new();
        buf.extend(self.kind.introducer().to_vec());
        if let Some(private) = &self.private {
            buf.push(private.into());
        }
        let mut i = 0usize;
        if self.kind == ControlFunctionKind::Osc
            && let Some(number) = &self.number
        {
            buf.extend(number);
            buf.push(*self.delimiter);
            i += 1;
        }
        if matches!(
            self.kind,
            ControlFunctionKind::C0 | ControlFunctionKind::Byte
        ) && let Some(code) = &self.code
        {
            buf.push(*code)
        }
        for param in params {
            if i > 0 {
                buf.push(*self.delimiter);
            }
            buf.extend(param);
            i += 1;
        }

        buf
    }

    /// Generate trie key using the first params alternative.
    ///
    /// The `param_marker` byte is used for CSI sequences:
    /// - 0x00 = no params
    /// - 0x01 = has params (normal case)
    /// - 0x02 + N = exactly N params (disambiguated sequences)
    ///
    /// The `has_data_params` flag indicates whether the struct has data
    /// parameters (fields in the data section). This is used to determine
    /// whether to include the data_delimiter after the static data string
    /// for OSC sequences.
    #[must_use]
    #[allow(dead_code)]
    pub fn get_key(
        &self,
        final_byte: Option<u8>,
        param_marker: u8,
        has_data_params: bool,
    ) -> Vec<u8> {
        self.get_key_with_params(
            final_byte,
            param_marker,
            has_data_params,
            self.params.first(),
        )
    }

    /// Generate trie key with a specific params set.
    #[must_use]
    pub fn get_key_with_params(
        &self,
        final_byte: Option<u8>,
        param_marker: u8,
        has_data_params: bool,
        params: &AnsiStrings,
    ) -> Vec<u8> {
        let mut buf: Vec<u8> = Vec::new();
        buf.extend(self.kind.introducer().to_vec());
        if let Some(private) = &self.private {
            buf.push(private.into());
        }
        if matches!(
            self.kind,
            ControlFunctionKind::C0 | ControlFunctionKind::Byte
        ) && let Some(code) = &self.code
        {
            // C0/Byte is ambiguous at '\x1b', so disambiguate with a `\0` prefix
            buf.push(0);
            buf.push(*code)
        }
        if matches!(self.kind, ControlFunctionKind::Csi,) {
            // Add param marker byte for disambiguation
            buf.push(param_marker);
        }
        // Include intermediate bytes for disambiguation (CSI, ESC, DCS)
        // Intermediate bytes come before the final byte in these sequences
        if matches!(
            self.kind,
            ControlFunctionKind::Csi
                | ControlFunctionKind::Esc
                | ControlFunctionKind::Dcs
        ) && let Some(intermediate) = &self.intermediate
        {
            buf.extend(intermediate.iter());
        }
        // Use provided final byte, or disambiguate by final byte itself (unless dynamic).
        if let Some(fb) = final_byte {
            buf.push(fb);
        } else if self.final_bytes.len() == 1 {
            buf.push(*self.final_bytes[0]);
        } else {
            buf.push(0);
        }
        let mut i = 0usize;
        if self.kind == ControlFunctionKind::Osc
            && let Some(number) = &self.number
        {
            buf.extend(number);
            buf.push(*self.delimiter);
            i += 1;
        }
        // Include OSC data field for disambiguation (e.g., "SetMark", "Copy=")
        if self.kind == ControlFunctionKind::Osc && !self.data.is_empty() {
            buf.extend(self.data.iter());
            // Include data_delimiter only if there are actual data params to follow
            // (e.g., '=' in "Copy=" when there's a field after it, but not for
            // unit structs like PromptStart where "A" is the entire data)
            if has_data_params {
                if let Some(data_delim) = &self.data_delimiter {
                    buf.push(data_delim.into());
                }
            }
        }
        // Include DCS data field for disambiguation (e.g., "m", " q", "\"p")
        if self.kind == ControlFunctionKind::Dcs && !self.data.is_empty() {
            buf.extend(self.data.iter());
        }
        for param in params {
            if i > 0 {
                buf.push(*self.delimiter);
            }
            buf.extend(param);
            i += 1;
        }

        buf
    }

    #[must_use]
    pub fn write_static_prefix(
        &self,
        writes: &mut WriteOperationBuilder,
    ) -> usize {
        let prefix = self.get_static_prefix();
        writes.write_bytes(&prefix);
        prefix.len()
    }

    #[must_use]
    pub fn get_static_suffix(&self) -> Option<Vec<u8>> {
        if self.final_bytes.len() > 1 {
            // Dynamic final byte
            return None;
        }

        let mut buf: Vec<u8> = Vec::new();
        if let Some(intermediate) = &self.intermediate {
            buf.extend(intermediate);
        }
        if !self.final_bytes.is_empty() {
            buf.push(*self.final_bytes[0]);
        }

        Some(buf)
    }

    #[must_use]
    pub fn write_static_suffix(
        &self,
        writes: &mut WriteOperationBuilder,
    ) -> usize {
        let mut counter = 0usize;
        if let Some(intermediate) = &self.intermediate {
            writes.write_bytes(intermediate);
            counter += intermediate.len();
        }

        if self.final_bytes.len() > 1 {
            writes.write_byte_expr(quote! {
                ::vtansi::encode::AnsiFinalByte::ansi_final_byte(self)
            });
        } else if self.final_bytes.len() == 1 {
            writes.write_byte(*self.final_bytes[0]);
        }
        counter += 1;

        counter
    }

    #[must_use]
    pub fn get_static_data(&self) -> Vec<u8> {
        self.data.to_vec()
    }

    #[must_use]
    pub fn write_static_data(
        &self,
        writes: &mut WriteOperationBuilder,
    ) -> usize {
        let data = self.get_static_data();
        writes.write_bytes(&data);
        data.len()
    }

    #[must_use]
    pub fn get_terminator(&self) -> Vec<u8> {
        self.kind.terminator().to_vec()
    }

    #[must_use]
    pub fn write_terminator(
        &self,
        writes: &mut WriteOperationBuilder,
    ) -> usize {
        let terminator = self.get_terminator();
        writes.write_bytes(&terminator);
        terminator.len()
    }
}

impl HasTypeProperties for DeriveInput {
    fn get_format_properties(
        &self,
        default_format: StructFormat,
        named: bool,
    ) -> syn::Result<FormatProperties> {
        let mut format = default_format;
        let mut delimiter = syn::parse_quote!(b';');

        let mut format_kw = None;
        let mut delimiter_kw = None;

        for meta in self.get_type_metadata()? {
            match meta {
                TypeMeta::Format { kw, format: fmt } => {
                    if let Some(fst_kw) = format_kw {
                        return Err(occurrence_error(fst_kw, kw, "format"));
                    }

                    format_kw = Some(kw);
                    format = fmt;

                    if format == StructFormat::Map && !named {
                        return Err(syn::Error::new_spanned(
                            kw,
                            "Tuple structs cannot use the `map` format. Use \
                             #[vtansi(format = \"vector\")] or omit the format attribute",
                        ));
                    }
                }
                TypeMeta::Delimiter {
                    kw,
                    delimiter: delim,
                } => {
                    if let Some(fst_kw) = delimiter_kw {
                        return Err(occurrence_error(fst_kw, kw, "delimiter"));
                    }

                    delimiter_kw = Some(kw);
                    delimiter = delim;
                }
                _ => {}
            }
        }

        Ok(FormatProperties { format, delimiter })
    }

    fn get_type_properties(&self) -> syn::Result<ValueProperties> {
        let repr_type = extract_repr_type(self);

        // Default format depends on struct type
        let default_format = match &self.data {
            Data::Struct(data) => match &data.fields {
                Fields::Unnamed(_) => StructFormat::Vector,
                _ => StructFormat::Map,
            },
            _ => StructFormat::Map,
        };

        let mut format = default_format;
        let mut delimiter = syn::parse_quote!(b';');
        let mut transparent = false;
        let mut encoded_len = None;
        let mut into = None;

        let mut format_kw = None;
        let mut delimiter_kw = None;
        let mut transparent_kw = None;
        let mut encoded_len_kw = None;
        let mut seen_into = None;

        for meta in self.get_type_metadata()? {
            match meta {
                TypeMeta::Format { kw, format: fmt } => {
                    if let Some(fst_kw) = format_kw {
                        return Err(occurrence_error(fst_kw, kw, "format"));
                    }

                    format_kw = Some(kw);
                    format = fmt;
                }
                TypeMeta::Delimiter {
                    kw,
                    delimiter: delim,
                } => {
                    if let Some(fst_kw) = delimiter_kw {
                        return Err(occurrence_error(fst_kw, kw, "delimiter"));
                    }

                    delimiter_kw = Some(kw);
                    delimiter = delim;
                }
                TypeMeta::Transparent { kw } => {
                    if let Some(fst_kw) = transparent_kw {
                        return Err(occurrence_error(
                            fst_kw,
                            kw,
                            "transparent",
                        ));
                    }

                    transparent_kw = Some(kw);
                    transparent = true;
                }
                TypeMeta::EncodedLen { kw, value } => {
                    if let Some(fst_kw) = encoded_len_kw {
                        return Err(occurrence_error(
                            fst_kw,
                            kw,
                            "encoded_len",
                        ));
                    }

                    encoded_len_kw = Some(kw);
                    encoded_len = Some(value);
                }
                TypeMeta::Into { kw, path } => {
                    if let Some(first_kw) = seen_into {
                        return Err(occurrence_error(first_kw, kw, "into"));
                    }
                    seen_into = Some(kw);
                    into = Some(path);
                }
                m => {
                    return Err(syn::Error::new(
                        m.span(),
                        "Control sequence attributes like #[vtansi(csi...)], #[vtansi(osc...)], etc. \
                         are not supported on the ToAnsi/FromAnsi derives.\n\
                         \n\
                         Use #[derive(AnsiControl)] instead for control sequences.\n\
                         \n\
                         Example migration:\n\
                         \n\
                         Before:\n\
                         #[derive(FromAnsi, ToAnsi)]\n\
                         #[vtansi(csi, finalbyte = 'H')]\n\
                         struct CursorPosition { row: u16, col: u16 }\n\
                         \n\
                         After:\n\
                         #[derive(AnsiControl)]\n\
                         #[vtansi(csi, finalbyte = 'H')]\n\
                         struct CursorPosition { row: u16, col: u16 }\n\
                         \n\
                         The AnsiControl derive automatically implements both FromAnsi and ToAnsi \
                         with proper control sequence framing.",
                    ));
                }
            }
        }

        // Validate transparent + Vec requires delimiter
        if transparent && let Data::Struct(data) = &self.data {
            let fields = match &data.fields {
                Fields::Named(fields) => {
                    fields.named.iter().collect::<Vec<_>>()
                }
                Fields::Unnamed(fields) => {
                    fields.unnamed.iter().collect::<Vec<_>>()
                }
                Fields::Unit => Vec::new(),
            };

            if fields.len() == 1 {
                let field = fields[0];
                let field_ty = &field.ty;

                if is_vec_type(field_ty) && delimiter_kw.is_none() {
                    return Err(syn::Error::new_spanned(
                        self,
                        "transparent structs wrapping Vec<T> require #[vtansi(delimiter = <bytelit>)] attribute\n\
                             \n\
                             Example:\n\
                             #[derive(FromAnsi, ToAnsi)]\n\
                             #[vtansi(transparent, delimiter = b';')]\n\
                             struct MyVec(Vec<u8>);",
                    ));
                }
            }
        }

        // Validate encoded_len is only used on enums without repr
        if let Some(kw) = encoded_len_kw {
            match &self.data {
                Data::Enum(_) => {
                    if repr_type.is_some() {
                        return Err(syn::Error::new_spanned(
                            kw,
                            "encoded_len attribute cannot be used on enums with #[repr(...)]\n\
                             \n\
                             Enums with #[repr(u8)] or similar already have their encoded length \
                             computed automatically from the integer type.\n\
                             \n\
                             Remove either the #[repr(...)] or the #[vtansi(encoded_len = ...)] attribute.",
                        ));
                    }
                }
                Data::Struct(_) => {
                    return Err(syn::Error::new_spanned(
                        kw,
                        "encoded_len attribute is not supported on structs\n\
                         \n\
                         This attribute is only valid for enums without #[repr(...)] attributes.",
                    ));
                }
                Data::Union(_) => {
                    return Err(syn::Error::new_spanned(
                        kw,
                        "encoded_len attribute is not supported on unions",
                    ));
                }
            }
        }

        Ok(ValueProperties {
            repr_type,
            format,
            delimiter,
            transparent,
            encoded_len,
            into,
        })
    }

    fn get_control_properties(
        &self,
        direction: ControlDirection,
    ) -> syn::Result<ControlProperties> {
        let mut format: Option<StructFormat> = None;
        let mut delimiter = syn::parse_quote!(b';');

        let mut format_kw = None;
        let mut delimiter_kw = None;

        // New control sequence properties
        let mut kind = None;
        let mut private = None;
        let mut params = AnsiStringsAlts::new();
        let mut intermediate = None;
        let mut final_bytes = AnsiChars::new();
        let mut data = AnsiString::new();
        let mut number = None;
        let mut data_delimiter = None;
        let mut code = None;
        let mut code_value_lit: Option<syn::LitInt> = None;
        let mut field_location = FieldLocation::Params;
        let mut into = None;

        // Track which control sequence attributes we've seen for duplicate
        // detection
        let mut seen_kind = None;
        let mut seen_private = None;
        let mut seen_params = None;
        let mut seen_intermediate = None;
        let mut seen_finalbyte = None;
        let mut seen_data = None;
        let mut seen_number = None;
        let mut seen_data_delimiter = None;
        let mut seen_code = None;
        let mut seen_location = None;
        let mut seen_into = None;
        let mut alias_of = None;
        let mut seen_alias_of = None;
        let mut disambiguate = false;
        let mut seen_disambiguate = None;

        for meta in self.get_type_metadata()? {
            match meta {
                TypeMeta::Format { kw, format: fmt } => {
                    if let Some(fst_kw) = format_kw {
                        return Err(occurrence_error(fst_kw, kw, "format"));
                    }

                    format_kw = Some(kw);
                    format = Some(fmt);
                }
                TypeMeta::Delimiter {
                    kw,
                    delimiter: delim,
                } => {
                    if let Some(fst_kw) = delimiter_kw {
                        return Err(occurrence_error(fst_kw, kw, "delimiter"));
                    }

                    delimiter_kw = Some(kw);
                    delimiter = delim;
                }
                TypeMeta::Kind { kw, kind: seq_kind } => {
                    if let Some(first_kw) = seen_kind {
                        return Err(duplicate_intro_error(first_kw, kw));
                    }
                    seen_kind = Some(kw);
                    kind = Some(seq_kind);
                }
                TypeMeta::Private { kw, value } => {
                    if let Some(first_kw) = seen_private {
                        return Err(occurrence_error(first_kw, kw, "private"));
                    }
                    seen_private = Some(kw);
                    private = Some(value);
                }
                TypeMeta::Params { kw, value } => {
                    if let Some(first_kw) = seen_params {
                        return Err(occurrence_error(first_kw, kw, "params"));
                    }
                    seen_params = Some(kw);
                    params = value;
                }
                TypeMeta::Intermediate { kw, value } => {
                    if let Some(first_kw) = seen_intermediate {
                        return Err(occurrence_error(
                            first_kw,
                            kw,
                            "intermediate",
                        ));
                    }
                    seen_intermediate = Some(kw);
                    intermediate = Some(value);
                }
                TypeMeta::FinalByte { kw, value } => {
                    if let Some(first_kw) = seen_finalbyte {
                        return Err(occurrence_error(
                            first_kw,
                            kw,
                            "finalbyte",
                        ));
                    }
                    seen_finalbyte = Some(kw);
                    final_bytes = value;
                }
                TypeMeta::Data { kw, value } => {
                    if let Some(first_kw) = seen_data {
                        return Err(occurrence_error(first_kw, kw, "data"));
                    }
                    seen_data = Some(kw);
                    data = value;
                }
                TypeMeta::Number { kw, value } => {
                    if let Some(first_kw) = seen_number {
                        return Err(occurrence_error(first_kw, kw, "number"));
                    }
                    seen_number = Some(kw);
                    number = Some(value);
                }
                TypeMeta::DataDelimiter { kw, delimiter } => {
                    if let Some(first_kw) = seen_data_delimiter {
                        return Err(occurrence_error(
                            first_kw,
                            kw,
                            "data_delimiter",
                        ));
                    }
                    seen_data_delimiter = Some(kw);
                    data_delimiter = Some(delimiter);
                }
                TypeMeta::Code { kw, value } => {
                    if let Some(first_kw) = seen_code {
                        return Err(occurrence_error(first_kw, kw, "code"));
                    }
                    seen_code = Some(kw);
                    // Store the literal for validation after we know the kind
                    code_value_lit = Some(value);
                }
                TypeMeta::FieldLocation { kw, location } => {
                    if let Some(first_kw) = seen_location {
                        return Err(occurrence_error(
                            first_kw,
                            kw,
                            "locate_all",
                        ));
                    }
                    seen_location = Some(kw);
                    field_location = location;
                }
                TypeMeta::Into { kw, path } => {
                    if let Some(first_kw) = seen_into {
                        return Err(occurrence_error(first_kw, kw, "into"));
                    }
                    seen_into = Some(kw);
                    into = Some(path);
                }
                TypeMeta::AliasOf { kw, path } => {
                    if let Some(first_kw) = seen_alias_of {
                        return Err(occurrence_error(first_kw, kw, "alias_of"));
                    }
                    seen_alias_of = Some(kw);
                    alias_of = Some(path);
                }
                TypeMeta::Disambiguate { kw } => {
                    if let Some(first_kw) = seen_disambiguate {
                        return Err(occurrence_error(
                            first_kw,
                            kw,
                            "disambiguate",
                        ));
                    }
                    seen_disambiguate = Some(kw);
                    disambiguate = true;
                }
                TypeMeta::Transparent { kw } => {
                    return Err(syn::Error::new_spanned(
                        kw,
                        "transparent attribute is not supported on control sequences",
                    ));
                }
                TypeMeta::EncodedLen { kw, .. } => {
                    return Err(syn::Error::new_spanned(
                        kw,
                        "encoded_len attribute is not supported on AnsiControl derive\n\
                         \n\
                         This attribute is only valid for ToAnsi derive on enums without #[repr(...)] attributes.",
                    ));
                }
            }
        }

        let kind = match kind {
            Some(kind) => kind,
            None => {
                return Err(syn::Error::new_spanned(
                    self,
                    "AnsiControl derive requires control sequence attributes.\n\
                     \n\
                     Add one of the following to your struct:\n\
                     - #[vtansi(csi, finalbyte = 'H')]        for CSI sequences\n\
                     - #[vtansi(osc, number = 0)]             for OSC sequences\n\
                     - #[vtansi(dcs, finalbyte = 'q')]        for DCS sequences\n\
                     - #[vtansi(esc)]                         for ESC sequences\n\
                     - #[vtansi(c0, code = 0x0E)]             for C0 control codes\n\
                     - #[vtansi(byte, code = 0xFF)]           for single byte codes (AnsiInput only)\n\
                     \n\
                     See the crate documentation for more examples.",
                ));
            }
        };

        // Validate the byte kind is only used with Input direction
        if kind == ControlFunctionKind::Byte
            && direction != ControlDirection::Input
        {
            return Err(syn::Error::new_spanned(
                self,
                "The 'byte' control function kind is only valid for AnsiInput.\n\
                 \n\
                 For AnsiOutput, use 'c0' instead if the code is in range 0x00-0x1F.",
            ));
        }

        // Validate the ss3 kind is only used with Input direction
        if kind == ControlFunctionKind::Ss3
            && direction != ControlDirection::Input
        {
            return Err(syn::Error::new_spanned(
                self,
                "The 'ss3' control function kind is only valid for AnsiInput.\n\
                 \n\
                 SS3 sequences (ESC O) are used for input parsing only.",
            ));
        }

        // Now validate the code value based on the kind
        if let Some(value_lit) = code_value_lit {
            match value_lit.base10_parse::<u8>() {
                Ok(val) => {
                    // For C0, enforce range restriction
                    if kind == ControlFunctionKind::C0 && val > 0x1F {
                        return Err(syn::Error::new_spanned(
                            value_lit,
                            "C0 code must be in range 0x00-0x1F",
                        ));
                    }
                    // For Byte, any u8 value is allowed
                    code = Some(val);
                }
                Err(e) => {
                    return Err(syn::Error::new_spanned(
                        value_lit,
                        format!("invalid code value: {}", e),
                    ));
                }
            }
        }

        let format = match format {
            Some(format) => format,
            None => StructFormat::Vector,
        };

        if seen_data_delimiter.is_none() {
            data_delimiter = Some(syn::parse_quote!(b';'));
        }

        // For SS3 and OSC sequences, default field location to Data if not explicitly set
        let field_location = if seen_location.is_none()
            && matches!(
                kind,
                ControlFunctionKind::Ss3 | ControlFunctionKind::Osc
            ) {
            FieldLocation::Data
        } else {
            field_location
        };

        Ok(ControlProperties {
            format,
            field_location,
            delimiter,
            kind,
            direction,
            private,
            params,
            intermediate,
            final_bytes,
            data,
            number,
            data_delimiter,
            code,
            into,
            alias_of,
            disambiguate,
        })
    }
}

/// Create an error for duplicate intro type declarations.
fn duplicate_intro_error(
    fst: ControlKeyword,
    snd: ControlKeyword,
) -> syn::Error {
    syn::Error::new(
        proc_macro2::Span::call_site(),
        format!(
            "found multiple occurrences of control function kind ({:?} and {:?})",
            fst, snd
        ),
    )
}
