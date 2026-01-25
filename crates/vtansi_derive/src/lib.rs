//! Derive macros for ANSI sequence encoding and decoding.
//!
//! This crate provides four derive macros:
//!
//! - **`FromAnsi`** - Parse ANSI parameter values into Rust types (also
//!   implements `TryFromAnsiIter` for vector format structs)
//! - **`ToAnsi`** - Encode Rust types into ANSI parameter values
//! - **`AnsiInput`** - Complete ANSI control sequences for input (terminal to host)
//! - **`AnsiOutput`** - Complete ANSI control sequences for output (host to terminal)
//!
//! # Quick Start
//!
//! ## Parameter Encoding/Decoding
//!
//! Use `FromAnsi` and `ToAnsi` for simple parameter values:
//!
//! ```ignore
//! #[derive(FromAnsi, ToAnsi)]
//! #[repr(u8)]
//! enum Color {
//!     Red = 0,
//!     Green = 1,
//!     Blue = 2,
//! }
//! ```
//!
//! ## Control Sequences
//!
//! Use `AnsiInput` or `AnsiOutput` for complete terminal control sequences:
//!
//! ```ignore
//! #[derive(AnsiOutput)]
//! #[vtansi(csi, finalbyte = 'H')]
//! struct CursorPosition {
//!     row: u16,
//!     col: u16,
//! }
//! // Encodes as: ESC [ 10 ; 20 H
//! ```
//!
//! # Features
//!
//! ## FromAnsi and ToAnsi
//!
//! - **Multiple parsing strategies** - supports integer-based (via
//!   `#[repr(...)]`), string-based, and structured field parsing
//! - **Default variants** - optionally specify a fallback variant for
//!   unrecognized enum values
//! - **Flexible struct formats** - encode/decode structs as `key=value` pairs
//!   or as positional values
//! - **Customizable delimiters** - configure field separators for structs
//! - **Optional fields** - automatic handling of `Option<T>` types
//! - **Parameter multiplexing** - multiple fields at same parameter position
//! - **Transparent Vec newtypes** - wrap `Vec<T>` with custom delimiters
//! - **Iterator-based parsing** - `FromAnsi` automatically implements
//!   `TryFromAnsiIter` for parsing from parameter iterators (vector format
//!   structs only; enums and map format structs require manual implementation)
//!
//! ## AnsiInput and AnsiOutput
//!
//! - **Complete control sequences** - CSI, OSC, DCS, ESC, C0, and more
//! - **Automatic trait implementation** - generates both FromAnsi and ToAnsi
//! - **Sequence validation** - compile-time checks for required attributes
//! - **Zero runtime overhead** - all code generated at compile time with
//!   `#[inline]` optimizations
//! - **Direction-specific** - `AnsiInput` for terminal-to-host sequences,
//!   `AnsiOutput` for host-to-terminal sequences
//!
//! # Enum Support
//!
//! For enums, the macros support both primitive representations and
//! string-based conversions:
//!
//! ```ignore
//! #[derive(FromAnsi, ToAnsi)]
//! #[repr(u8)]
//! enum Color {
//!     Red = 0,
//!     Green = 1,
//!     Blue = 2,
//! }
//! ```
//!
//! # Struct Support
//!
//! For structs with named or unnamed (tuple) fields, the macros support two
//! formats:
//!
//! ## Key-Value Format (default)
//!
//! ```ignore
//! #[derive(FromAnsi, ToAnsi)]
//! struct Settings {
//!     width: u32,
//!     height: u32,
//! }
//! // Encodes as: "width=800;height=600"
//! ```
//!
//! Note: Key-value format is only available for named fields, not tuple
//! structs.
//!
//! ## Vector Format
//!
//! ```ignore
//! #[derive(FromAnsi, ToAnsi)]
//! #[vtansi(format = "vector")]
//! struct Point {
//!     x: i32,
//!     y: i32,
//! }
//! // Encodes as: "100;200"
//! ```
//!
//! ## Tuple Structs (automatic vector format)
//!
//! ```ignore
//! #[derive(FromAnsi, ToAnsi)]
//! struct Point(i32, i32);
//! // Automatically uses vector format
//! // Encodes as: "100;200"
//! ```
//!
//! Tuple structs automatically default to `vector` format and cannot use
//! the `map` format.
//!
//! ## Custom Delimiter
//!
//! ```ignore
//! #[derive(FromAnsi, ToAnsi)]
//! #[vtansi(delimiter = ",")]
//! struct Config {
//!     name: String,
//!     value: u32,
//! }
//! // Encodes as: "name=foo,value=42"
//! ```
//!
//! ## Optional Fields
//!
//! ### Key-Value Format
//!
//! ```ignore
//! #[derive(FromAnsi, ToAnsi)]
//! struct Settings {
//!     width: u32,
//!     height: u32,
//!     title: Option<String>,  // Optional field
//! }
//! // Parses: "width=800;height=600" (title is None)
//! // Parses: "width=800;height=600;title=MyApp" (title is Some("MyApp"))
//! ```
//!
//! Fields with `Option<T>` type are automatically optional in `map` format.
//! Missing optional fields will be set to `None` instead of causing a parse
//! error.
//!
//! ### Vector Format
//!
//! ```ignore
//! #[derive(FromAnsi, ToAnsi)]
//! #[vtansi(format = "vector")]
//! struct Point3D {
//!     x: i32,
//!     y: i32,
//!     z: Option<i32>,  // Optional trailing field
//! }
//! // Encodes: Point3D { x: 10, y: 20, z: Some(30) } -> "10;20;30"
//! // Encodes: Point3D { x: 10, y: 20, z: None } -> "10;20"
//! // Parses: "10;20" -> Point3D { x: 10, y: 20, z: None }
//! // Parses: "10;20;30" -> Point3D { x: 10, y: 20, z: Some(30) }
//! ```
//!
//! In vector format, `Option<T>` fields must appear after all non-optional
//! fields. Trailing `None` values are omitted during encoding. During parsing,
//! missing trailing fields are set to `None`.
//!
//! Multiple optional fields are supported:
//!
//! ```ignore
//! #[derive(FromAnsi, ToAnsi)]
//! struct Coordinates(i32, i32, Option<i32>, Option<i32>);
//! // Parses: "10;20" -> all optional fields are None
//! // Parses: "10;20;30" -> third field is Some(30), fourth is None
//! // Parses: "10;20;;40" -> third field is None, fourth is Some(40)
//! // Parses: "10;20;30;40" -> both optional fields are Some
//! ```
//!
//! ## Skipping Fields
//!
//! ```ignore
//! #[derive(FromAnsi, ToAnsi)]
//! struct Data {
//!     id: u32,
//!     #[vtansi(skip)]
//!     internal: String,
//! }
//! // Only 'id' is encoded/decoded
//! ```
//!
//! ## Parameter Multiplexing
//!
//! The `#[vtansi(muxwith = field)]` attribute enables multiplexing/demultiplexing
//! where multiple fields can encode to and decode from the same parameter
//! position. This is useful for bit-field extraction, modifier flags, or
//! any case where multiple pieces of information are packed into a single
//! parameter.
//!
//! Fields marked with `muxwith`:
//! - **During encoding**: Contribute to the specified field's parameter via
//!   the `AnsiMuxEncode` trait
//! - **During decoding**: Parse from the same parameter position via
//!   `TryFromAnsi`
//!
//! This allows multiple fields to work together at the same parameter position,
//! with each field's type defining how it contributes to encoding and extracts
//! information during decoding.
//!
//! ```ignore
//! // Example: Mouse event with button code and modifiers
//! #[derive(FromAnsi, ToAnsi)]
//! #[vtansi(format = "vector")]
//! struct MouseParam {
//!     button_code: u8,           // Encodes button + modifiers together
//!     #[vtansi(muxwith = "button_code")]
//!     modifiers: Modifiers,      // Extracts modifier bits from same position
//! }
//! // Encodes: "26" (right button=2 + ctrl=16 + alt=8)
//! // Decodes: "26" -> MouseParam {
//! //     button_code: 26,
//! //     modifiers: Modifiers { ctrl: true, alt: true, ... }
//! // }
//! ```
//!
//! ### Advanced Usage (AnsiMuxEncode Trait)
//!
//! For proper bidirectional multiplexing where fields can be modified
//! independently and encoding stays fresh, implement the `AnsiMuxEncode` trait
//! on the multiplexed field type:
//!
//! ```ignore
//! use vtansi::encode::AnsiMuxEncode;
//!
//! // Button provides the base value (0, 1, 2 for left, middle, right)
//! impl ToAnsi for Button {
//!     fn to_ansi(&self) -> impl AnsiEncode {
//!         self.code() // Returns 0, 1, or 2
//!     }
//! }
//!
//! // Modifiers contribute by setting bits on top of the button code
//! impl AnsiMuxEncode for Modifiers {
//!     type BaseType = u8;
//!
//!     fn mux_encode(&self, base: u8) -> u8 {
//!         let mut result = base;
//!         if self.shift { result |= 4; }
//!         if self.alt { result |= 8; }
//!         if self.ctrl { result |= 16; }
//!         result
//!     }
//! }
//!
//! #[derive(FromAnsi, ToAnsi)]
//! #[vtansi(format = "vector")]
//! struct MouseParam {
//!     button: Button,           // Provides base encoding (0-2)
//!     #[vtansi(muxwith = "button")]
//!     modifiers: Modifiers,     // Contributes via AnsiMuxEncode
//! }
//!
//! // Encoding: button=2, shift=true, ctrl=true
//! // → 2 | 4 | 16 = 22
//! // Output: "22"
//!
//! // Decoding: "22"
//! // → button extracts: 22 & !0x1C = 2 (right button)
//! // → modifiers extracts: shift=true, ctrl=true from bits
//!
//! // Modifying fields automatically updates encoding - no stale state!
//! param.modifiers.alt = true;  // Encoding now produces "30"
//! ```
//!
//! This feature only works with `vector` format structs.
//!
//! # Transparent Attribute
//!
//! The `#[vtansi(transparent)]` attribute allows creating newtype wrappers
//! around a single field, delegating encoding/decoding to that field's
//! implementation.
//!
//! ## Transparent Vec Support
//!
//! When using `transparent` with a `Vec<T>` field, you must specify a
//! delimiter using `#[vtansi(delimiter = <bytelit>)]`. The elements will be
//! encoded/decoded with the delimiter between them.
//!
//! ```ignore
//! #[derive(FromAnsi, ToAnsi)]
//! #[vtansi(transparent, delimiter = b';')]
//! struct Numbers(Vec<u32>);
//!
//! // Encodes: Numbers(vec![1, 2, 3]) -> "1;2;3"
//! // Decodes: "1;2;3" -> Numbers(vec![1, 2, 3])
//! ```
//!
//! This works with any type `T` that implements `TryFromAnsi` and `AnsiEncode`:
//!
//! ```ignore
//! #[derive(FromAnsi, ToAnsi)]
//! #[vtansi(transparent, delimiter = b',')]
//! struct Tags(Vec<String>);
//!
//! // Encodes: Tags(vec!["foo".into(), "bar".into()]) -> "foo,bar"
//! ```
//!
//! **Note**: The delimiter attribute is required when using `transparent` with
//! `Vec<T>`. Omitting it will result in a compile-time error.
//!
//! # Control Sequences with AnsiInput and AnsiOutput
//!
//! The `AnsiInput` and `AnsiOutput` derive macros generate complete ANSI
//! control sequences. Use these for terminal control sequences (CSI, OSC,
//! DCS, etc.) that need proper escape sequence framing.
//!
//! - **`AnsiInput`** - For sequences sent from terminal to host (e.g., key
//!   events, mouse events)
//! - **`AnsiOutput`** - For sequences sent from host to terminal (e.g., cursor
//!   control, colors)
//!
//! **Important**: `FromAnsi` and `ToAnsi` are for parameter encoding only.
//! Control sequence attributes like `#[vtansi(csi...)]` must use
//! `#[derive(AnsiInput)]` or `#[derive(AnsiOutput)]` instead.
//!
//! ## CSI Sequences (Control Sequence Introducer)
//!
//! CSI sequences start with `ESC [` and end with a final byte. They're used
//! for cursor control, graphics rendition, and other terminal functions.
//!
//! ```ignore
//! #[derive(AnsiOutput)]
//! #[vtansi(csi, finalbyte = 'H')]
//! struct CursorPosition {
//!     row: u16,
//!     col: u16,
//! }
//! // Encodes as: ESC [ row ; col H
//! // Example: ESC [ 10 ; 20 H (move cursor to row 10, column 20)
//! ```
//!
//! Private CSI sequences (using private marker bytes):
//!
//! ```ignore
//! #[derive(AnsiOutput)]
//! #[vtansi(csi, private = '?', finalbyte = 'h')]
//! struct SetMode {
//!     mode: u16,
//! }
//! // Encodes as: ESC [ ? mode h
//! // Example: ESC [ ? 1049 h (enable alternate screen buffer)
//! ```
//!
//! ## OSC Sequences (Operating System Command)
//!
//! OSC sequences start with `ESC ]` and are used for setting window titles,
//! colors, and other terminal properties.
//!
//! ```ignore
//! #[derive(AnsiOutput)]
//! #[vtansi(osc, number = "0")]
//! struct SetWindowTitle {
//!     title: String,
//! }
//! // Encodes as: ESC ] 0 ; title ST
//! // Example: ESC ] 0 ; My Window ST
//! ```
//!
//! ## DCS Sequences (Device Control String)
//!
//! DCS sequences start with `ESC P` and are used for complex device control.
//!
//! ```ignore
//! #[derive(AnsiOutput)]
//! #[vtansi(dcs, finalbyte = 'q', data = "$q")]
//! struct RequestStatusString {
//!     status_type: String,
//! }
//! // Encodes as: ESC P finalbyte data status_type ST
//! ```
//!
//! ## SS3 Sequences (Single Shift 3) - AnsiInput only
//!
//! SS3 sequences have the format `ESC O <data>` where 'O' is the finalbyte
//! of the ESC sequence itself. They are used for parsing input sequences like
//! application cursor keys and function keys. SS3 sequences must either define
//! static data or have exactly one field (which defaults to data location).
//!
//! ### Static Data (No Fields)
//!
//! For unit structs with static data:
//!
//! ```ignore
//! #[derive(AnsiInput)]
//! #[vtansi(ss3, data = "A")]
//! struct SS3CursorUp;
//! // Parses: ESC O A
//! ```
//!
//! ```ignore
//! #[derive(AnsiInput)]
//! #[vtansi(ss3, data = "P")]
//! struct SS3FunctionKey1;
//! // Parses: ESC O P
//! ```
//!
//! ### Single Field (Dynamic Data)
//!
//! When a struct has a single field, it captures the data byte(s) after ESC O:
//!
//! ```ignore
//! #[derive(AnsiInput)]
//! #[vtansi(ss3)]
//! struct SS3Key {
//!     key: u8,
//! }
//! // Parses: ESC O <key>
//! // Example: ESC O A (key = b'A')
//! ```
//!
//! **Note:** SS3 sequences are restricted to AnsiInput only, as they are
//! primarily used for parsing keyboard input in application cursor mode.
//! ## ESC Sequences (Simple Escape Sequences)
//!
//! Simple escape sequences start with `ESC` followed by a final byte.
//!
//! ```ignore
//! #[derive(AnsiOutput)]
//! #[vtansi(esc, finalbyte = '7')]
//! struct SaveCursor;
//! // Encodes as: ESC 7
//! ```
//!
//! ## C0 Control Codes
//!
//! C0 codes are single-byte control characters (0x00-0x1F).
//!
//! ```ignore
//! #[derive(AnsiOutput)]
//! #[vtansi(c0, code = 0x0E)]
//! struct ShiftOut;
//! // Encodes as: 0x0E (single byte)
//! ```
//!
//! ## Single Byte Codes (Input Only)
//!
//! For `AnsiInput` only, the `byte` control function kind allows any single
//! byte value (0x00-0xFF) without the C0 range restriction.
//!
//! ```ignore
//! #[derive(AnsiInput)]
//! #[vtansi(byte, code = 0xFF)]
//! struct CustomInputByte;
//! // Parses: 0xFF (single byte, any value allowed)
//! ```
//!
//! Note: The `byte` kind is only valid for `AnsiInput`. For `AnsiOutput`,
//! use `c0` if the code is in range 0x00-0x1F.
//!
//! ## Available Framing Attributes
//!
//! ### Sequence Types
//!
//! - `csi` - CSI sequence (ESC [)
//! - `esc` - Simple ESC sequence
//! - `escst` - ESC sequence with String Terminator (ESC ... ST)
//! - `osc` - Operating System Command (ESC ])
//! - `dcs` - Device Control String (ESC P)
//! - `ss3` - Single Shift 3 (ESC O, AnsiInput only)
//! - `c0` - C0 control code (0x00-0x1F)
//! - `byte` - Single byte code (0x00-0xFF, AnsiInput only)
//!
//! ### Required Attributes
//!
//! - `finalbyte = 'X'` - Required for CSI and DCS sequences. The final
//!   byte that terminates the sequence. Can be a single character or multiple
//!   alternatives using `|` operator (e.g., `finalbyte = 'h' | 'l'`).
//!   Not used for SS3 (where 'O' is the finalbyte).
//! - `code = 0xNN` - Required for C0 and byte sequences. The control code byte.
//!   For C0, must be in range 0x00-0x1F. For byte (AnsiInput only), can be any
//!   value 0x00-0xFF.
//!
//! ### Optional Attributes
//!
//! - `private = '?'` - Private marker byte for CSI sequences (e.g., '?', '>',
//!   '=').
//! - `intermediate = ' '` - Intermediate bytes (max 2 bytes). Can be a single
//!   character or string.
//! - `params = ["6", "1"]` - Static parameter values for unit structs (const
//!   sequences). Can use `|` operator for alternatives that map to the same
//!   handler (e.g., `params = ['12'] | ['13']`).
//! - `data = "$q"` - Static data string for DCS, OSC, and SS3 sequences.
//!   For SS3, this is the data following `ESC O`.
//! - `number = "133"` - OSC numeric parameter (Ps in ESC ] Ps ; Pt ST).
//! - `locate_all = "params" | "data"` - Default location for all fields.
//!
//! ## Unit Structs (Constant Sequences)
//!
//! For sequences with no variable parts, use unit structs with constant
//! parameters:
//!
//! ```ignore
//! #[derive(AnsiOutput)]
//! #[vtansi(csi, params = ["6"], finalbyte = 'n')]
//! struct ReportCursorPosition;
//! // Encodes as: ESC [ 6 n (Device Status Report)
//! ```
//!
//! ## AnsiInput/AnsiOutput vs FromAnsi/ToAnsi
//!
//! - **`AnsiInput`/`AnsiOutput`**: Complete terminal control sequences with
//!   escape codes and framing. Use for CSI, OSC, DCS, ESC, C0, and other
//!   control sequences. Automatically implements both FromAnsi and ToAnsi
//!   traits.
//!
//! - **`FromAnsi`/`ToAnsi`**: Parameter encoding/decoding only (no escape
//!   codes). Use for enums and structs that represent parameter values within
//!   sequences or for testing.
//!
//! Control sequence attributes (csi, osc, dcs, etc.) can only be used with
//! `AnsiInput` or `AnsiOutput`, not with `FromAnsi` or `ToAnsi`.

#![recursion_limit = "128"]
#![forbid(unsafe_code)]

extern crate proc_macro;

mod helpers;
mod macros;

use proc_macro::TokenStream;
use syn::DeriveInput;

/// Derive macro for `FromAnsi` trait.
///
/// This macro generates implementations of both `TryFromAnsi` and `TryFromAnsiIter`
/// traits (for vector format structs). The `TryFromAnsi` trait parses from a
/// byte slice, while `TryFromAnsiIter` parses from an iterator of byte slices
/// (useful for consuming parameters from a pre-split parameter list).
///
/// This macro can be applied to:
///
/// ## Enums
///
/// Enums that either:
/// 1. Have a primitive integer representation (e.g., `#[repr(u8)]`)
/// 2. Implement `std::convert::TryFrom<&str>`
///
/// ## Structs
///
/// Structs with named or unnamed (tuple) fields where each field implements
/// `TryFromAnsi`. Tuple structs automatically use `vector` format.
///
/// # Generated Traits
///
/// ## `TryFromAnsi`
///
/// Parses the type from a single byte slice. For structs, the byte slice is
/// split by the delimiter to extract field values.
///
/// ## `TryFromAnsiIter`
///
/// Parses the type from an iterator of byte slices. This trait is **only**
/// automatically derived for vector format structs:
///
/// - **Vector format structs**: Automatically derived; consumes one iterator
///   item per field
/// - **Map format structs**: **Not automatically derived**; must be
///   implemented manually
/// - **Enums**: **Not automatically derived**; must be implemented manually
/// - **Transparent structs**: Behavior depends on the inner type
///
/// # Attributes
///
/// ## Enum Attributes
///
/// - `#[vtansi(default)]` - Mark a variant as the default fallback for
///   unrecognized values. Only one variant can be marked as default.
///   The default variant can be:
///   - A unit variant (returns a constant value)
///   - A tuple variant with one field (captures the unrecognized value)
///
/// ## Struct Attributes
///
/// - `#[vtansi(format = "vector")]` - Use vector format (values only) instead of
///   key=value map format.
/// - `#[vtansi(delimiter = ",")]` - Custom field delimiter (default: ";").
///
/// ## Field Attributes
///
/// - `#[vtansi(skip)]` - Skip this field during encoding/decoding.
/// - `#[vtansi(muxwith = field)]` - (Structs only, vector format) Multiplex this field
///   with another field. During encoding, the field contributes to
///   parameter N via `AnsiMuxEncode::mux_encode()`. During decoding, the field
///   parses from parameter N via `TryFromAnsi`. This enables packing multiple
///   pieces of information into a single parameter (e.g., button code with
///   modifier flags). The field's type must implement both `TryFromAnsi` and
///   `AnsiMuxEncode` (see `vtansi::encode::AnsiMuxEncode`).
///
/// ## Framing Attributes (Structs only)
///
/// For complete control sequence generation, use framing attributes to specify
/// the sequence type and properties:
///
/// - `#[vtansi(csi, finalbyte = 'H')]` - CSI sequence (ESC [)
/// - `#[vtansi(esc, finalbyte = '7')]` - Simple ESC sequence
/// - `#[vtansi(osc, number = "0")]` - OSC sequence (ESC ])
/// - `#[vtansi(dcs, finalbyte = 'q')]` - DCS sequence (ESC P)
/// - `#[vtansi(c0, code = 0x0E)]` - C0 control code
///
/// See the crate-level documentation for complete framing documentation.
///
/// # Examples
///
/// ## Unit default variant
///
/// ```ignore
/// #[derive(FromAnsi)]
/// #[repr(u8)]
/// enum Color {
///     Red = 0,
///     Green = 1,
///     Blue = 2,
///     #[vtansi(default)]
///     Unknown = 255,
/// }
/// ```
///
/// ## Capturing default variant
///
/// ```ignore
/// #[derive(FromAnsi)]
/// #[repr(u8)]
/// enum StatusCode {
///     Ok = 200,
///     NotFound = 404,
///     #[vtansi(default)]
///     Unknown(u8),  // Captures unrecognized codes
/// }
/// ```
///
/// ## String-based enum with capturing default
///
/// ```ignore
/// #[derive(FromAnsi)]
/// enum Command {
///     Quit,
///     Save,
///     #[vtansi(default)]
///     Custom(String),  // Captures unrecognized commands
/// }
///
/// impl TryFrom<&str> for Command {
///     type Error = String;
///     fn try_from(s: &str) -> Result<Self, Self::Error> {
///         // implementation
///     }
/// }
/// ```
#[proc_macro_derive(FromAnsi, attributes(vtansi))]
pub fn derive_from_ansi(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);

    let toks = macros::from_ansi::from_ansi_inner(&ast)
        .unwrap_or_else(|err| err.to_compile_error());
    helpers::debug_print_generated(&ast, &toks);
    toks.into()
}

/// Derive macro for `ToAnsi` trait.
///
/// This macro can be applied to:
///
/// ## Enums
///
/// Enums that either:
/// 1. Have a primitive integer representation (e.g., `#[repr(u8)]`)
/// 2. Implement `AsRef<str>`
///
/// ## Structs
///
/// Structs with named or unnamed (tuple) fields where each field implements
/// `ToAnsi`. Tuple structs automatically use `vector` format.
///
/// # Framing Support
///
/// This macro also supports generating complete control sequences when
/// framing attributes are specified on structs. See the crate-level
/// documentation section "Framed Sequences (Control Sequences)" for details.
///
/// # Examples
///
/// For enums with primitive representations:
///
/// ```ignore
/// #[derive(ToAnsi)]
/// #[repr(u8)]
/// enum Color {
///     Red = 0,
///     Green = 1,
///     Blue = 2,
/// }
/// ```
///
/// For enums implementing `AsRef<str>`:
///
/// ```ignore
/// #[derive(ToAnsi)]
/// enum Mode {
///     Normal,
///     Insert,
/// }
///
/// impl AsRef<str> for Mode {
///     fn as_ref(&self) -> &str {
///         // implementation
///     }
/// }
/// ```
#[proc_macro_derive(ToAnsi, attributes(vtansi))]
pub fn derive_to_ansi(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);

    let toks = macros::to_ansi::to_ansi_inner(&ast)
        .unwrap_or_else(|err| err.to_compile_error());
    helpers::debug_print_generated(&ast, &toks);
    toks.into()
}

/// Derive macro for `AnsiInput` - complete ANSI control sequences for input.
///
/// This macro generates both `TryFromAnsi` and `ToAnsi` implementations for
/// ANSI control sequences (CSI, OSC, DCS, ESC, C0, etc.) that represent
/// terminal-to-host input sequences (e.g., key events, mouse events).
///
/// # Control Sequence Types
///
/// The macro supports all standard ANSI control sequence types:
///
/// - **CSI** (Control Sequence Introducer) - `ESC [` sequences
/// - **OSC** (Operating System Command) - `ESC ]` sequences
/// - **DCS** (Device Control String) - `ESC P` sequences
/// - **ESC** - Plain escape sequences
/// - **SS3** - Single Shift 3 sequences (AnsiInput only)
/// - **C0** - Control characters (0x00-0x1F)
/// - **Byte** - Single byte codes (0x00-0xFF, AnsiInput only)
///
/// # Required Attributes
///
/// Each control sequence type requires specific attributes:
///
/// - CSI: `#[vtansi(csi, finalbyte = 'X')]`
/// - OSC: `#[vtansi(osc, number = "N")]`
/// - DCS: `#[vtansi(dcs, finalbyte = 'X')]`
/// - ESC: `#[vtansi(esc, finalbyte = 'X')]`
/// - SS3: `#[vtansi(ss3)]` with optional `data = "X"`
/// - C0: `#[vtansi(c0, code = 0xNN)]`
/// - Byte: `#[vtansi(byte, code = 0xNN)]`
///
/// # Examples
///
/// ## CSI Sequence
///
/// ```ignore
/// #[derive(AnsiInput)]
/// #[vtansi(csi, finalbyte = 'R')]
/// struct CursorPositionReport {
///     row: u16,
///     col: u16,
/// }
///
/// // Parses: ESC [ row ; col R
/// // Example: ESC [ 10 ; 20 R
/// ```
///
/// ## SS3 Sequence (Input Only)
///
/// ```ignore
/// #[derive(AnsiInput)]
/// #[vtansi(ss3, data = "A")]
/// struct SS3CursorUp;
///
/// // Parses: ESC O A
/// ```
///
/// ## Single Byte Code (Input Only)
///
/// ```ignore
/// #[derive(AnsiInput)]
/// #[vtansi(byte, code = 0xFF)]
/// struct CustomInputByte;
///
/// // Parses: 0xFF (any byte value allowed for AnsiInput)
/// ```
///
/// # Comparison with ToAnsi/FromAnsi
///
/// - Use `ToAnsi`/`FromAnsi` for simple parameter encoding/decoding
/// - Use `AnsiInput`/`AnsiOutput` for complete ANSI control sequences with framing
///
/// The `AnsiInput` derive automatically handles:
/// - Sequence framing (introducer, parameters, final byte, terminator)
/// - Parameter encoding/decoding
/// - Both encoding (ToAnsi) and decoding (FromAnsi) in one derive
///
/// # Errors
///
/// The macro will produce a compile error if:
/// - Applied to an enum or union (only structs are supported)
/// - No framing attributes are specified
/// - Required attributes for the sequence type are missing
#[proc_macro_derive(AnsiInput, attributes(vtansi))]
pub fn derive_ansi_control_input(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);

    let toks = macros::ansi_control::ansi_control_inner(
        &ast,
        helpers::ControlDirection::Input,
    )
    .unwrap_or_else(|err| err.to_compile_error());
    helpers::debug_print_generated(&ast, &toks);
    toks.into()
}

/// Derive macro for `AnsiOutput` - complete ANSI control sequences for output.
///
/// This macro generates both `TryFromAnsi` and `ToAnsi` implementations for
/// ANSI control sequences (CSI, OSC, DCS, ESC, C0, etc.) that represent
/// host-to-terminal output sequences (e.g., cursor control, colors, screen manipulation).
///
/// See [`AnsiInput`](derive_ansi_control_input) for detailed documentation on
/// control sequence types and attributes. The main difference is:
///
/// - **`AnsiOutput`** - For sequences sent from host to terminal
/// - **`AnsiInput`** - For sequences received from terminal (includes SS3 and byte sequences)
///
/// # Example
///
/// ```ignore
/// #[derive(AnsiOutput)]
/// #[vtansi(csi, finalbyte = 'H')]
/// struct CursorPosition {
///     row: u16,
///     col: u16,
/// }
///
/// // Encodes as: ESC [ row ; col H
/// // Example: ESC [ 10 ; 20 H
/// ```
#[proc_macro_derive(AnsiOutput, attributes(vtansi))]
pub fn derive_ansi_control_output(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);

    let toks = macros::ansi_control::ansi_control_inner(
        &ast,
        helpers::ControlDirection::Output,
    )
    .unwrap_or_else(|err| err.to_compile_error());
    helpers::debug_print_generated(&ast, &toks);
    toks.into()
}
