//! Metadata parsing utilities for vtansi attributes.
//!
//! This module provides the core infrastructure for parsing `#[vtansi(...)]`
//! attributes on enum variants, structs, and fields. It defines custom
//! keywords, metadata types, and extension traits for extracting structured
//! metadata from syntax tree nodes.

use quote::quote;
use syn::{
    DeriveInput, Field, LitInt, LitStr, Token, Variant,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
};

/// Custom keywords for vtansi attributes.
///
/// These keywords are recognized within `#[vtansi(...)]` attribute lists and
/// are parsed using syn's `custom_keyword!` macro.
pub mod kw {
    use syn::custom_keyword;
    use syn::spanned::Spanned;

    custom_keyword!(alias_of);
    custom_keyword!(code);
    custom_keyword!(data);
    custom_keyword!(data_delimiter);
    custom_keyword!(disambiguate);
    custom_keyword!(into);
    custom_keyword!(locate_all);
    custom_keyword!(locate);
    custom_keyword!(default);
    custom_keyword!(delimiter);
    custom_keyword!(encoded_len);
    custom_keyword!(finalbyte);
    custom_keyword!(flatten);
    custom_keyword!(format);
    custom_keyword!(intermediate);
    custom_keyword!(muxwith);
    custom_keyword!(number);
    custom_keyword!(params);
    custom_keyword!(private);
    custom_keyword!(skip);
    custom_keyword!(transparent);

    macro_rules! control_keywords {
        ( $( $V:ident($t:ident) ),+ $(,)? ) => {
            $( custom_keyword!($t); )+

            #[derive(Debug, Clone, PartialEq, Eq)]
            pub enum ControlKeyword {
                $( $V($t), )+
            }

            impl ControlKeyword {
                pub fn span(&self) -> proc_macro2::Span {
                    match self {
                        $( Self::$V(v) => v.span(), )+
                    }
                }
            }
        };
    }

    control_keywords! {
        Byte(byte),
        C0(c0),
        Csi(csi),
        Dcs(dcs),
        Esc(esc),
        EscSt(escst),
        Osc(osc),
        Ss3(ss3),
    }
}

pub use kw::ControlKeyword;

/// Where field is located in the control sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldLocation {
    /// Parameter section of the sequence
    Params,
    /// Static params section - field captures from position 0 of all_params
    /// (including the static params consumed by trie matching)
    StaticParams,
    /// Data section of the sequence (e.g in DCS)
    Data,
    /// Final byte (e.g in ESC G0/G1 designators)
    Final,
}

impl Parse for FieldLocation {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let s: LitStr = input.parse()?;
        match s.value().as_str() {
            "params" => Ok(FieldLocation::Params),
            "static_params" => Ok(FieldLocation::StaticParams),
            "data" => Ok(FieldLocation::Data),
            "final" => Ok(FieldLocation::Final),
            _ => Err(syn::Error::new_spanned(
                s,
                "location must be \"params\", \"static_params\", \"data\", or \"final\"",
            )),
        }
    }
}

/// Format style for struct encoding/decoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructFormat {
    /// Encode as `key=value` pairs (default).
    Map,
    /// Encode as values only, in field order.
    Vector,
}

impl Parse for StructFormat {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let s: LitStr = input.parse()?;
        match s.value().as_str() {
            "map" => Ok(StructFormat::Map),
            "vector" => Ok(StructFormat::Vector),
            _ => Err(syn::Error::new_spanned(
                s,
                "format must be either \"map\" or \"vector\"",
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::AsRefStr)]
pub enum ControlDirection {
    /// Terminal to host input (e.g key events)
    Input,
    /// Host to terminal output (i.e render sequences, reports etc.)
    Output,
}

impl ControlDirection {
    pub fn as_lib_enum(&self) -> proc_macro2::TokenStream {
        let variant =
            syn::Ident::new(self.as_ref(), proc_macro2::Span::call_site());
        quote!(::vtansi::AnsiControlDirection::#variant)
    }
}

/// Control function type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::AsRefStr)]
pub enum ControlFunctionKind {
    /// Single byte control code (any value, AnsiInput only)
    Byte,
    /// C0 control character (0x00-0x1F)
    C0,
    /// CSI (Control Sequence Introducer) - ESC [
    Csi,
    /// OSC (Operating System Command) - ESC ]
    Osc,
    /// DCS (Device Control String) - ESC P
    Dcs,
    /// ESC - Other escape sequence (e.g DECKPAM)
    Esc,
    /// ESC ... ST - Escape sequence terminated with ESC \
    /// This serves as a catch-all with less-common and less-defined
    /// sequences such as APC, PM and SOS.
    EscSt,
    /// SS3 (Single Shift 3) - ESC O (AnsiInput only)
    Ss3,
}

impl ControlFunctionKind {
    /// Fixed byte sequence introducing this control function kind (if any)
    #[inline]
    pub fn introducer(&self) -> &'static [u8] {
        match self {
            Self::Byte | Self::C0 => b"",
            Self::Csi => b"\x1B[",
            Self::Osc => b"\x1B]",
            Self::Dcs => b"\x1BP",
            Self::Esc | Self::EscSt => b"\x1B",
            Self::Ss3 => b"\x1BO",
        }
    }

    /// Fixed byte sequence terminating this control function kind (if any)
    #[inline]
    pub fn terminator(&self) -> &'static [u8] {
        match self {
            Self::Dcs | Self::Osc | Self::EscSt => b"\x1B\\",
            _ => b"",
        }
    }

    pub fn name(&self) -> String {
        self.as_ref().to_string()
    }

    pub fn as_lib_enum(&self) -> proc_macro2::TokenStream {
        let variant =
            syn::Ident::new(&self.name(), proc_macro2::Span::call_site());
        quote!(::vtansi::AnsiControlFunctionKind::#variant)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AnsiChar(u8);

impl AnsiChar {
    pub fn to_literal(&self) -> syn::LitByte {
        syn::LitByte::new(self.0, proc_macro2::Span::call_site())
    }
}

impl std::ops::Deref for AnsiChar {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for AnsiChar {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::convert::From<AnsiChar> for u8 {
    fn from(value: AnsiChar) -> Self {
        value.0
    }
}

impl std::convert::From<&AnsiChar> for u8 {
    fn from(value: &AnsiChar) -> Self {
        value.0
    }
}

impl Parse for AnsiChar {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        use syn::Lit::{self, *};

        let lit: Lit = input.parse()?;
        let bytes = match lit {
            Char(c) if c.value().is_ascii() => c.value() as u8,
            Byte(b) => b.value(),
            other => {
                return Err(syn::Error::new_spanned(
                    other,
                    "expected ASCII char, or byte literal",
                ));
            }
        };

        Ok(Self(bytes))
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AnsiChars(Vec<AnsiChar>);

impl AnsiChars {
    pub fn new() -> Self {
        Self(Vec::new())
    }
}

impl std::ops::Deref for AnsiChars {
    type Target = Vec<AnsiChar>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for AnsiChars {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a> IntoIterator for &'a AnsiChars {
    type Item = &'a AnsiChar;
    type IntoIter = std::slice::Iter<'a, AnsiChar>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a> IntoIterator for &'a mut AnsiChars {
    type Item = &'a mut AnsiChar;
    type IntoIter = std::slice::IterMut<'a, AnsiChar>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

impl IntoIterator for AnsiChars {
    type Item = AnsiChar;
    type IntoIter = std::vec::IntoIter<AnsiChar>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Parse for AnsiChars {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let elems: Punctuated<AnsiChar, Token![|]> =
            Punctuated::parse_separated_nonempty(input)?;
        Ok(Self(elems.into_iter().collect::<Vec<AnsiChar>>()))
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AnsiString(Vec<u8>);

impl AnsiString {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn to_literal(&self) -> syn::LitByteStr {
        syn::LitByteStr::new(&self.0, proc_macro2::Span::call_site())
    }
}

impl std::ops::Deref for AnsiString {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for AnsiString {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a> IntoIterator for &'a AnsiString {
    type Item = &'a u8;
    type IntoIter = std::slice::Iter<'a, u8>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a> IntoIterator for &'a mut AnsiString {
    type Item = &'a mut u8;
    type IntoIter = std::slice::IterMut<'a, u8>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

impl IntoIterator for AnsiString {
    type Item = u8;
    type IntoIter = std::vec::IntoIter<u8>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Parse for AnsiString {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        use syn::Lit::{self, *};

        let lit: Lit = input.parse()?;
        let bytes = match lit {
            Str(s) => s.value().as_bytes().to_vec(),
            ByteStr(bs) => bs.value().to_vec(),
            Char(c) => {
                let mut buf = [0u8; 4];
                c.value().encode_utf8(&mut buf).as_bytes().to_vec()
            }
            Byte(b) => vec![b.value()],
            other => {
                return Err(syn::Error::new_spanned(
                    other,
                    "expected string, byte string, char, or byte literal",
                ));
            }
        };

        Ok(Self(bytes))
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AnsiStrings(Vec<AnsiString>);

impl AnsiStrings {
    pub fn new() -> Self {
        Self(Vec::new())
    }
}

impl std::ops::Deref for AnsiStrings {
    type Target = Vec<AnsiString>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for AnsiStrings {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a> IntoIterator for &'a AnsiStrings {
    type Item = &'a AnsiString;
    type IntoIter = std::slice::Iter<'a, AnsiString>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a> IntoIterator for &'a mut AnsiStrings {
    type Item = &'a mut AnsiString;
    type IntoIter = std::slice::IterMut<'a, AnsiString>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

impl IntoIterator for AnsiStrings {
    type Item = AnsiString;
    type IntoIter = std::vec::IntoIter<AnsiString>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Parse for AnsiStrings {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        syn::bracketed!(content in input);
        let elems: syn::punctuated::Punctuated<AnsiString, Token![,]> =
            content.parse_terminated(AnsiString::parse, Token![,])?;
        Ok(Self(elems.into_iter().collect::<Vec<AnsiString>>()))
    }
}

/// A collection of alternative `AnsiStrings` separated by `|`.
///
/// This represents alternative static param sets like `params = ['12'] | ['13']`.
/// Each alternative is an `AnsiStrings` (a bracketed list of strings).
#[derive(Debug, Default, Clone)]
pub struct AnsiStringsAlts(Vec<AnsiStrings>);

impl AnsiStringsAlts {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Returns true if there are no alternatives (empty).
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the number of alternatives.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns the first alternative, or an empty `AnsiStrings` if none.
    pub fn first(&self) -> &AnsiStrings {
        static EMPTY: std::sync::LazyLock<AnsiStrings> =
            std::sync::LazyLock::new(AnsiStrings::new);
        self.0.first().unwrap_or(&EMPTY)
    }

    /// Iterate over all alternatives.
    pub fn iter(&self) -> impl Iterator<Item = &AnsiStrings> {
        self.0.iter()
    }
}

impl std::ops::Deref for AnsiStringsAlts {
    type Target = Vec<AnsiStrings>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for AnsiStringsAlts {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a> IntoIterator for &'a AnsiStringsAlts {
    type Item = &'a AnsiStrings;
    type IntoIter = std::slice::Iter<'a, AnsiStrings>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a> IntoIterator for &'a mut AnsiStringsAlts {
    type Item = &'a mut AnsiStrings;
    type IntoIter = std::slice::IterMut<'a, AnsiStrings>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

impl IntoIterator for AnsiStringsAlts {
    type Item = AnsiStrings;
    type IntoIter = std::vec::IntoIter<AnsiStrings>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Parse for AnsiStringsAlts {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let elems: Punctuated<AnsiStrings, Token![|]> =
            Punctuated::parse_separated_nonempty(input)?;
        Ok(Self(elems.into_iter().collect::<Vec<AnsiStrings>>()))
    }
}

/// Metadata that can be attached to types (enums or structs).
///
/// This enum represents all possible metadata items that can appear in a
/// `#[vtansi(...)]` attribute on a type definition.
pub enum TypeMeta {
    /// `#[vtansi(format = "map")]` - specify struct encoding format.
    Format {
        kw: kw::format,
        format: StructFormat,
    },
    /// `#[vtansi(delimiter = b';')]` - specify field delimiter for structs.
    Delimiter {
        kw: kw::delimiter,
        delimiter: AnsiChar,
    },
    /// Sequence kind (csi, esc, osc, etc).
    Kind {
        kw: kw::ControlKeyword,
        kind: ControlFunctionKind,
    },
    /// `#[vtansi(private = '?')]` - private marker byte.
    Private { kw: kw::private, value: AnsiChar },
    /// `#[vtansi(params = ["6", "1"])]` - constant parameter sequences.
    /// Supports alternatives: `params = ['12'] | ['13']`.
    Params {
        kw: kw::params,
        value: AnsiStringsAlts,
    },
    /// `#[vtansi(intermediate = ' ')]` - intermediate byte sequence.
    Intermediate {
        kw: kw::intermediate,
        value: AnsiString,
    },
    /// `#[vtansi(finalbyte = 'h')]` - final byte(s).
    FinalByte { kw: kw::finalbyte, value: AnsiChars },
    /// `#[vtansi(data = "...")]` - static data string.
    Data { kw: kw::data, value: AnsiString },
    /// `#[vtansi(number = "1")]` - OSC numeric parameter.
    Number { kw: kw::number, value: AnsiString },
    /// `#[vtansi(data_sep = ";")]` - separator between static data and
    /// first field.
    DataDelimiter {
        kw: kw::data_delimiter,
        delimiter: AnsiChar,
    },
    /// `#[vtansi(code = 0x0E)]` - C0 control code.
    Code { kw: kw::code, value: syn::LitInt },
    /// `#[vtansi(transparent)]` - encode struct transparently as its single field.
    Transparent { kw: kw::transparent },
    /// `#[vtansi(encoded_len = N)]` - constant encoded length in bytes.
    EncodedLen {
        kw: kw::encoded_len,
        value: syn::LitInt,
    },
    /// `#[vtansi(locate_all = "param" | "data")]` - specify default location
    /// for all fields.
    ///
    /// Individual fields can opt out using `#[vtansi(locate = "param" | "data")]`.
    FieldLocation {
        kw: kw::locate_all,
        location: FieldLocation,
    },
    /// `#[vtansi(into = path::to::Type)]` - custom coercion target.
    Into { kw: kw::into, path: syn::Path },
    /// `#[vtansi(alias_of = PrimaryType)]` - mark this type as an alias.
    ///
    /// Alias types encode to the same byte sequence as the primary type but
    /// do not register in the parser trie. This is useful when multiple types
    /// represent the same byte sequence (e.g., CtrlJ and EnterKey both map to 0x0A).
    AliasOf { kw: kw::alias_of, path: syn::Path },
    /// `#[vtansi(disambiguate)]` - use exact param count for trie key disambiguation.
    ///
    /// This is used to disambiguate CSI sequences that share the same final byte
    /// but differ in parameter count. For example, ScrollDown (0-1 params) and
    /// TrackMouse (5 params) both use final byte 'T'.
    ///
    /// When marked with `disambiguate`, the sequence:
    /// - Must not have optional parameters (all fields must be required)
    /// - Uses `2 + total_params` as the key byte instead of the boolean `has_params`
    ///
    /// The parser will first try the normal `has_params` lookup, and if that fails,
    /// it will backtrack and try `2 + param_count` to match disambiguated sequences.
    Disambiguate { kw: kw::disambiguate },
}

impl TypeMeta {
    pub fn span(&self) -> proc_macro2::Span {
        match self {
            Self::Format { kw, .. } => kw.span(),
            Self::Delimiter { kw, .. } => kw.span(),
            Self::Kind { kw, .. } => kw.span(),
            Self::Private { kw, .. } => kw.span(),
            Self::Params { kw, .. } => kw.span(),
            Self::Intermediate { kw, .. } => kw.span(),
            Self::FinalByte { kw, .. } => kw.span(),
            Self::Data { kw, .. } => kw.span(),
            Self::Number { kw, .. } => kw.span(),
            Self::DataDelimiter { kw, .. } => kw.span(),
            Self::Code { kw, .. } => kw.span(),
            Self::Transparent { kw } => kw.span(),
            Self::EncodedLen { kw, .. } => kw.span(),
            Self::FieldLocation { kw, .. } => kw.span(),
            Self::Into { kw, .. } => kw.span(),
            Self::AliasOf { kw, .. } => kw.span(),
            Self::Disambiguate { kw } => kw.span(),
        }
    }
}

impl Parse for TypeMeta {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::byte) {
            let kw = input.parse::<kw::byte>()?;
            let kw = ControlKeyword::Byte(kw);
            let kind = ControlFunctionKind::Byte;
            Ok(TypeMeta::Kind { kw, kind })
        } else if lookahead.peek(kw::c0) {
            let kw = input.parse::<kw::c0>()?;
            let kw = ControlKeyword::C0(kw);
            let kind = ControlFunctionKind::C0;
            Ok(TypeMeta::Kind { kw, kind })
        } else if lookahead.peek(kw::csi) {
            let kw = input.parse::<kw::csi>()?;
            let kw = ControlKeyword::Csi(kw);
            let kind = ControlFunctionKind::Csi;
            Ok(TypeMeta::Kind { kw, kind })
        } else if lookahead.peek(kw::osc) {
            let kw = input.parse::<kw::osc>()?;
            let kw = ControlKeyword::Osc(kw);
            let kind = ControlFunctionKind::Osc;
            Ok(TypeMeta::Kind { kw, kind })
        } else if lookahead.peek(kw::dcs) {
            let kw = input.parse::<kw::dcs>()?;
            let kw = ControlKeyword::Dcs(kw);
            let kind = ControlFunctionKind::Dcs;
            Ok(TypeMeta::Kind { kw, kind })
        } else if lookahead.peek(kw::esc) {
            let kw = input.parse::<kw::esc>()?;
            let kw = ControlKeyword::Esc(kw);
            let kind = ControlFunctionKind::Esc;
            Ok(TypeMeta::Kind { kw, kind })
        } else if lookahead.peek(kw::escst) {
            let kw = input.parse::<kw::escst>()?;
            let kw = ControlKeyword::EscSt(kw);
            let kind = ControlFunctionKind::EscSt;
            Ok(TypeMeta::Kind { kw, kind })
        } else if lookahead.peek(kw::ss3) {
            let kw = input.parse::<kw::ss3>()?;
            let kw = ControlKeyword::Ss3(kw);
            let kind = ControlFunctionKind::Ss3;
            Ok(TypeMeta::Kind { kw, kind })
        } else if lookahead.peek(kw::format) {
            let kw = input.parse::<kw::format>()?;
            input.parse::<Token![=]>()?;
            let format = input.parse()?;
            Ok(TypeMeta::Format { kw, format })
        } else if lookahead.peek(kw::locate_all) {
            let kw = input.parse::<kw::locate_all>()?;
            input.parse::<Token![=]>()?;
            let location = input.parse()?;
            Ok(TypeMeta::FieldLocation { kw, location })
        } else if lookahead.peek(kw::delimiter) {
            let kw = input.parse::<kw::delimiter>()?;
            input.parse::<Token![=]>()?;
            let delimiter = input.parse()?;
            Ok(TypeMeta::Delimiter { kw, delimiter })
        } else if lookahead.peek(kw::into) {
            let kw = input.parse::<kw::into>()?;
            input.parse::<Token![=]>()?;
            let path = input.parse()?;
            Ok(TypeMeta::Into { kw, path })
        } else if lookahead.peek(kw::private) {
            let kw = input.parse::<kw::private>()?;
            input.parse::<Token![=]>()?;
            let value = input.parse()?;
            Ok(TypeMeta::Private { kw, value })
        } else if lookahead.peek(kw::params) {
            let kw = input.parse::<kw::params>()?;
            input.parse::<Token![=]>()?;
            let value = input.parse()?;
            Ok(TypeMeta::Params { kw, value })
        } else if lookahead.peek(kw::intermediate) {
            let kw = input.parse::<kw::intermediate>()?;
            input.parse::<Token![=]>()?;
            let value = input.parse()?;
            Ok(TypeMeta::Intermediate { kw, value })
        } else if lookahead.peek(kw::finalbyte) {
            let kw = input.parse::<kw::finalbyte>()?;
            input.parse::<Token![=]>()?;
            let value = input.parse()?;
            Ok(TypeMeta::FinalByte { kw, value })
        } else if lookahead.peek(kw::data) {
            let kw = input.parse::<kw::data>()?;
            input.parse::<Token![=]>()?;
            let value = input.parse()?;
            Ok(TypeMeta::Data { kw, value })
        } else if lookahead.peek(kw::number) {
            let kw = input.parse::<kw::number>()?;
            input.parse::<Token![=]>()?;
            let value = input.parse()?;
            Ok(TypeMeta::Number { kw, value })
        } else if lookahead.peek(kw::data_delimiter) {
            let kw = input.parse::<kw::data_delimiter>()?;
            input.parse::<Token![=]>()?;
            let delimiter = input.parse()?;
            Ok(TypeMeta::DataDelimiter { kw, delimiter })
        } else if lookahead.peek(kw::code) {
            let kw = input.parse::<kw::code>()?;
            input.parse::<Token![=]>()?;
            let value = input.parse()?;
            Ok(TypeMeta::Code { kw, value })
        } else if lookahead.peek(kw::transparent) {
            let kw = input.parse::<kw::transparent>()?;
            Ok(TypeMeta::Transparent { kw })
        } else if lookahead.peek(kw::encoded_len) {
            let kw = input.parse::<kw::encoded_len>()?;
            input.parse::<Token![=]>()?;
            let value = input.parse()?;
            Ok(TypeMeta::EncodedLen { kw, value })
        } else if lookahead.peek(kw::alias_of) {
            let kw = input.parse::<kw::alias_of>()?;
            input.parse::<Token![=]>()?;
            let path = input.parse()?;
            Ok(TypeMeta::AliasOf { kw, path })
        } else if lookahead.peek(kw::disambiguate) {
            let kw = input.parse::<kw::disambiguate>()?;
            Ok(TypeMeta::Disambiguate { kw })
        } else {
            Err(lookahead.error())
        }
    }
}

/// Wrapper for parsing multiple comma-separated TypeMeta items.
struct TypeMetaList {
    items: Vec<TypeMeta>,
}

impl Parse for TypeMetaList {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let items = syn::punctuated::Punctuated::<TypeMeta, Token![,]>::parse_terminated(input)?;
        Ok(TypeMetaList {
            items: items.into_iter().collect(),
        })
    }
}

/// Metadata that can be attached to enum variants.
///
/// This enum represents all possible metadata items that can appear in a
/// `#[vtansi(...)]` attribute on an enum variant. Each variant captures the
/// keyword token for use in error reporting.
pub enum VariantMeta {
    /// `#[vtansi(default)]` - mark a variant as the default fallback.
    ///
    /// When present, this variant will be used when parsing fails to match
    /// any other variant.
    Default { kw: kw::default },
}

impl Parse for VariantMeta {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::default) {
            let kw = input.parse::<kw::default>()?;
            Ok(VariantMeta::Default { kw })
        } else {
            Err(lookahead.error())
        }
    }
}

/// Metadata that can be attached to struct fields.
///
/// This enum represents all possible metadata items that can appear in a
/// `#[vtansi(...)]` attribute on a struct field.
pub enum FieldMeta {
    /// `#[vtansi(skip)]` - skip this field during encoding/decoding.
    Skip { kw: kw::skip },
    /// `#[vtansi(muxwith = "name" | index)]` - multiplex this field with
    /// another.  During encoding, contributes to parameter specified
    /// via AnsiMuxEncode trait.  During decoding, parses from parameter
    /// specified.
    Mux { kw: kw::muxwith, field: syn::Member },
    /// `#[vtansi(param)]` - indicate that this field should be encoded into/
    /// parsed from the params section (overrides type-level data_fields).
    Location {
        kw: kw::locate,
        location: FieldLocation,
    },
    /// `#[vtansi(flatten)]` - flatten this field, delegating parameter
    /// iterator parsing to the field's `try_from_ansi_iter` method.
    Flatten { kw: kw::flatten },
}

impl Parse for FieldMeta {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::skip) {
            let kw = input.parse::<kw::skip>()?;
            Ok(FieldMeta::Skip { kw })
        } else if lookahead.peek(kw::locate) {
            let kw = input.parse::<kw::locate>()?;
            input.parse::<Token![=]>()?;
            let location = input.parse()?;
            Ok(FieldMeta::Location { kw, location })
        } else if lookahead.peek(kw::muxwith) {
            let kw = input.parse::<kw::muxwith>()?;
            input.parse::<Token![=]>()?;
            let field = if input.peek(LitStr) {
                let s: LitStr = input.parse()?;
                syn::Member::Named(syn::Ident::new(&s.value(), s.span()))
            } else if input.peek(LitInt) {
                let n: LitInt = input.parse()?;
                syn::Member::Unnamed(syn::Index {
                    index: n.base10_parse::<u32>()?,
                    span: n.span(),
                })
            } else {
                return Err(syn::Error::new(
                    input.span(),
                    r#"expected integer or string literal (e.g. `muxwith = 0` or `muxwith = "a"`) "#,
                ));
            };
            Ok(FieldMeta::Mux { kw, field })
        } else if lookahead.peek(kw::flatten) {
            let kw = input.parse::<kw::flatten>()?;
            Ok(FieldMeta::Flatten { kw })
        } else {
            Err(lookahead.error())
        }
    }
}

/// Extension trait for parsing vtansi metadata from variants.
///
/// This trait extends `syn::Variant` with a method to extract all
/// `#[vtansi(...)]` attributes and parse them into structured metadata.
pub trait VariantExt {
    /// Extract all vtansi metadata from a variant's attributes.
    ///
    /// This method scans the variant's attributes, filters for those with
    /// the `vtansi` identifier, and parses their contents into
    /// `VariantMeta` values.
    ///
    /// # Errors
    ///
    /// Return an error if any attribute cannot be parsed according to the
    /// `VariantMeta` grammar.
    fn get_metadata(&self) -> syn::Result<Vec<VariantMeta>>;
}

impl VariantExt for Variant {
    fn get_metadata(&self) -> syn::Result<Vec<VariantMeta>> {
        let mut result = Vec::new();

        for attr in &self.attrs {
            if !attr.path().is_ident("vtansi") {
                continue;
            }

            let meta = attr.parse_args::<VariantMeta>()?;
            result.push(meta);
        }

        Ok(result)
    }
}

/// Extension trait for parsing vtansi metadata from types.
///
/// This trait extends `syn::DeriveInput` with a method to extract all
/// `#[vtansi(...)]` attributes and parse them into structured metadata.
pub trait DeriveInputExt {
    /// Extract all vtansi metadata from a type's attributes.
    ///
    /// This method scans the type's attributes, filters for those with the
    /// `vtansi` identifier, and parses their contents into `TypeMeta`
    /// values.
    ///
    /// # Errors
    ///
    /// Return an error if any attribute cannot be parsed according to the
    /// `TypeMeta` grammar.
    fn get_type_metadata(&self) -> syn::Result<Vec<TypeMeta>>;
}

impl DeriveInputExt for DeriveInput {
    fn get_type_metadata(&self) -> syn::Result<Vec<TypeMeta>> {
        let mut result = Vec::new();

        for attr in &self.attrs {
            if !attr.path().is_ident("vtansi") {
                continue;
            }

            let meta_list = attr.parse_args::<TypeMetaList>()?;
            result.extend(meta_list.items);
        }

        Ok(result)
    }
}

/// Extension trait for parsing vtansi metadata from fields.
///
/// This trait extends `syn::Field` with a method to extract all
/// `#[vtansi(...)]` attributes and parse them into structured metadata.
pub trait FieldExt {
    /// Extract all vtansi metadata from a field's attributes.
    ///
    /// This method scans the field's attributes, filters for those with the
    /// `vtansi` identifier, and parses their contents into `FieldMeta`
    /// values.
    ///
    /// # Errors
    ///
    /// Return an error if any attribute cannot be parsed according to the
    /// `FieldMeta` grammar.
    fn get_metadata(&self) -> syn::Result<Vec<FieldMeta>>;
}

impl FieldExt for Field {
    fn get_metadata(&self) -> syn::Result<Vec<FieldMeta>> {
        let mut result = Vec::new();

        for attr in &self.attrs {
            if !attr.path().is_ident("vtansi") {
                continue;
            }

            let meta = attr.parse_args::<FieldMeta>()?;
            result.push(meta);
        }

        Ok(result)
    }
}
