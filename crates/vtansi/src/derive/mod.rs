//! Derive macros and documentation for `#[vtansi(...)]` attributes.
//!
//! This module re-exports the derive macros from `vtansi_derive` and provides
//! documentation anchors for IDE hover support.

// Allow non-standard naming for doc anchors (they must match attribute names)
#![allow(non_upper_case_globals)]

/// Marker type representing a Rust type path in attribute arguments.
///
/// Used for documentation purposes in attributes like `into` and `alias_of`
/// that accept type paths.
pub struct Type;

// Re-export derive macros
pub use vtansi_derive::{AnsiInput, AnsiOutput, FromAnsi, ToAnsi};

// =============================================================================
// VTANSI ATTRIBUTE ANCHOR
// =============================================================================

/// The `#[vtansi(...)]` helper attribute for configuring ANSI sequence derive macros.
///
/// This attribute is used with `FromAnsi`, `ToAnsi`, `AnsiInput`, and `AnsiOutput`
/// derive macros to configure how Rust types are encoded to and decoded from
/// ANSI escape sequences.
///
/// # Usage
///
/// ```ignore
/// #[derive(AnsiOutput)]
/// #[vtansi(csi, finalbyte = 'H')]
/// struct CursorPosition {
///     row: u16,
///     col: u16,
/// }
/// // Encodes as: ESC [ row ; col H
/// ```
///
/// # Sequence Types
///
/// Specify the sequence type as the first argument:
///
/// | Type    | Description                          | Prefix    |
/// |---------|--------------------------------------|-----------|
/// | `csi`   | Control Sequence Introducer          | `ESC [`   |
/// | `osc`   | Operating System Command             | `ESC ]`   |
/// | `dcs`   | Device Control String                | `ESC P`   |
/// | `esc`   | Simple escape sequence               | `ESC`     |
/// | `escst` | Escape with String Terminator        | `ESC...ST`|
/// | `ss3`   | Single Shift 3 (input only)          | `ESC O`   |
/// | `c0`    | C0 control code                      | (none)    |
/// | `byte`  | Single byte (input only)             | (none)    |
///
/// # Type-Level Attributes
///
/// - `finalbyte = 'X'` - Final byte for CSI/DCS sequences
/// - `private = '?'` - Private marker for CSI sequences
/// - `params = ["1", "2"]` - Static parameters
/// - `intermediate = ' '` - Intermediate bytes
/// - `data = "..."` - Static data string
/// - `number = "N"` - OSC number
/// - `code = 0xNN` - C0/byte code value
/// - `format = "map" | "vector"` - Struct encoding format
/// - `delimiter = b';'` - Field delimiter
/// - `transparent` - Transparent wrapper
/// - `into = Type` - Custom target type
/// - `alias_of = Type` - Type alias (no registry entry)
///
/// # Field-Level Attributes
///
/// - `skip` - Skip field during encoding/decoding
/// - `muxwith = field` - Multiplex with another field
/// - `locate = "params" | "data" | "final"` - Field location in sequence
/// - `flatten` - Flatten nested structure
///
/// # Variant-Level Attributes
///
/// - `default` - Default variant for unrecognized values
///
/// Hover over individual keywords for detailed documentation.
pub fn vtansi() {}

// =============================================================================
// CONTROL SEQUENCE TYPES
// =============================================================================

/// CSI (Control Sequence Introducer) sequence marker.
///
/// Marks a struct as a CSI sequence, which starts with `ESC [` and ends with a
/// final byte.
///
/// # Required Attributes
///
/// - `finalbyte = 'X'` - The final byte that terminates the sequence
///
/// # Optional Attributes
///
/// - `private = '?'` - Private marker byte (`?`, `>`, `<`, `=`)
/// - `intermediate = ' '` - Intermediate bytes (max 2)
/// - `params = ["1", "2"]` - Static parameters for unit structs
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
/// // Encodes as: ESC [ row ; col H
/// ```
///
/// # Private CSI Sequences
///
/// ```ignore
/// #[derive(AnsiOutput)]
/// #[vtansi(csi, private = '?', finalbyte = 'h')]
/// struct SetMode {
///     mode: u16,
/// }
/// // Encodes as: ESC [ ? mode h
/// ```
pub fn csi(_: CsiArgs) {}

/// Arguments for CSI sequences.
#[allow(dead_code)]
pub struct CsiArgs {
    /// Final byte that terminates the sequence.
    ///
    /// Required for CSI sequences. Specifies the character that ends
    /// the parameter list and defines the command.
    ///
    /// # Syntax
    ///
    /// Single character:
    /// ```ignore
    /// #[vtansi(csi, finalbyte = 'H')]
    /// ```
    ///
    /// Multiple alternatives (using `|` operator):
    /// ```ignore
    /// #[vtansi(csi, finalbyte = 'h' | 'l')]
    /// ```
    ///
    /// # Valid Range
    ///
    /// Final bytes are typically in the range 0x40-0x7E (`@` through `~`).
    pub finalbyte: char,

    /// Private marker byte for CSI sequences.
    ///
    /// Specifies a private-use marker that appears after `ESC [` but before
    /// parameters. Common values are `?`, `>`, `<`, and `=`.
    ///
    /// # Example
    ///
    /// ```ignore
    /// #[derive(AnsiOutput)]
    /// #[vtansi(csi, private = '?', finalbyte = 'h')]
    /// struct DecSetMode { mode: u16 }
    /// // Encodes as: ESC [ ? mode h
    /// ```
    ///
    /// # Common Uses
    ///
    /// - `?` - DEC private modes (e.g., `ESC [ ? 1049 h` for alternate screen)
    /// - `>` - Secondary device attributes
    /// - `=` - Tertiary device attributes
    pub private: char,

    /// Static parameter values.
    ///
    /// Specifies constant parameters for unit structs that have no variable data.
    /// Each parameter is a string that will be encoded as-is.
    ///
    /// # Syntax
    ///
    /// ```ignore
    /// #[vtansi(csi, params = ["6"], finalbyte = 'n')]
    /// struct ReportCursorPosition;
    /// // Encodes as: ESC [ 6 n
    /// ```
    ///
    /// Alternative parameters (using `|` operator):
    /// ```ignore
    /// #[vtansi(csi, params = ["12"] | ["13"], finalbyte = 'n')]
    /// struct DeviceStatusReport;
    /// // Matches both: ESC [ 12 n  and  ESC [ 13 n
    /// ```
    pub params: &'static [&'static str],

    /// Intermediate bytes in the sequence.
    ///
    /// Specifies intermediate bytes that appear between parameters and the
    /// final byte. Maximum of 2 intermediate bytes are allowed.
    ///
    /// # Syntax
    ///
    /// Single character:
    /// ```ignore
    /// #[vtansi(csi, intermediate = ' ', finalbyte = 'q')]
    /// ```
    ///
    /// Multiple characters (as string):
    /// ```ignore
    /// #[vtansi(csi, intermediate = " $", finalbyte = 'p')]
    /// ```
    pub intermediate: &'static str,
}

/// Anchor value for CSI argument field access.
pub const CSI_ARGS: CsiArgs = CsiArgs {
    finalbyte: '\0',
    private: '\0',
    params: &[],
    intermediate: "",
};

/// OSC (Operating System Command) sequence marker.
///
/// Marks a struct as an OSC sequence, which starts with `ESC ]` and ends with
/// the String Terminator (ST).
///
/// # Optional Attributes
///
/// - `number = "N"` - The OSC number (Ps in `ESC ] Ps ; Pt ST`)
/// - `data = "..."` - Static data string
/// - `data_delimiter = ";"` - Custom separator (default: `;`)
///
/// Note: All fields in OSC sequences default to the `data` location.
///
/// # Example
///
/// ```ignore
/// #[derive(AnsiOutput)]
/// #[vtansi(osc, number = "0")]
/// struct SetWindowTitle {
///     title: String,
/// }
/// // Encodes as: ESC ] 0 ; title ST
/// ```
pub fn osc(_: OscArgs) {}

/// Arguments for OSC sequences.
#[allow(dead_code)]
pub struct OscArgs {
    /// OSC numeric parameter (Ps).
    ///
    /// Optional. Specifies the numeric command identifier that appears after
    /// `ESC ]` and before the semicolon separator. If not provided, the
    /// sequence matches on just the introducer.
    ///
    /// # Common OSC Numbers
    ///
    /// - `0` - Set icon name and window title
    /// - `1` - Set icon name
    /// - `2` - Set window title
    /// - `4` - Set/query color palette
    /// - `52` - Clipboard operations
    /// - `133` - Shell integration (prompt markers)
    ///
    /// # Example
    ///
    /// ```ignore
    /// #[derive(AnsiOutput)]
    /// #[vtansi(osc, number = "0")]
    /// struct SetTitle { title: String }
    /// // Encodes as: ESC ] 0 ; title ST
    /// ```
    pub number: &'static str,

    /// Static data string.
    ///
    /// Specifies constant data that appears after the number and separator.
    pub data: &'static str,

    /// Static parameter values.
    pub params: &'static [&'static str],

    /// Custom separator between static data and parameters.
    ///
    /// Overrides the default `;` separator used between the static data/number
    /// and the first dynamic parameter.
    pub data_delimiter: &'static str,
}

/// Anchor value for OSC argument field access.
pub const OSC_ARGS: OscArgs = OscArgs {
    number: "",
    data: "",
    params: &[],
    data_delimiter: "",
};

/// DCS (Device Control String) sequence marker.
///
/// Marks a struct as a DCS sequence, which starts with `ESC P` and ends with
/// the String Terminator (ST).
///
/// # Required Attributes
///
/// - `finalbyte = 'X'` - The final byte after the introducer
///
/// # Optional Attributes
///
/// - `data = "..."` - Static data string
/// - `intermediate = ' '` - Intermediate bytes
/// - `params = ["1"]` - Static parameters
///
/// # Example
///
/// ```ignore
/// #[derive(AnsiOutput)]
/// #[vtansi(dcs, finalbyte = 'q', data = "$q")]
/// struct RequestStatusString {
///     status_type: String,
/// }
/// ```
pub fn dcs(_: DcsArgs) {}

/// Arguments for DCS sequences.
#[allow(dead_code)]
pub struct DcsArgs {
    /// Final byte that terminates the sequence.
    pub finalbyte: char,

    /// Static data string.
    pub data: &'static str,

    /// Intermediate bytes.
    pub intermediate: &'static str,

    /// Static parameter values.
    pub params: &'static [&'static str],

    /// Custom separator between static data and parameters.
    pub data_delimiter: &'static str,
}

/// Anchor value for DCS argument field access.
pub const DCS_ARGS: DcsArgs = DcsArgs {
    finalbyte: '\0',
    data: "",
    intermediate: "",
    params: &[],
    data_delimiter: "",
};

/// Simple ESC sequence marker.
///
/// Marks a struct as a simple escape sequence that starts with `ESC` followed
/// by a final byte. These are typically single-character commands.
///
/// # Optional Attributes
///
/// - `finalbyte = 'X'` - The character following ESC
///
/// # Example
///
/// ```ignore
/// #[derive(AnsiOutput)]
/// #[vtansi(esc, finalbyte = '7')]
/// struct SaveCursor;
/// // Encodes as: ESC 7
/// ```
pub fn esc(_: EscArgs) {}

/// Arguments for ESC sequences.
#[allow(dead_code)]
pub struct EscArgs {
    /// Final byte that follows ESC.
    pub finalbyte: char,

    /// Intermediate bytes.
    pub intermediate: &'static str,
}

/// Anchor value for ESC argument field access.
pub const ESC_ARGS: EscArgs = EscArgs {
    finalbyte: '\0',
    intermediate: "",
};

/// ESC...ST (Escape with String Terminator) sequence marker.
///
/// Marks a struct as an escape sequence that ends with a String Terminator.
/// This is used for less common sequences like APC, PM, and SOS.
///
/// # Example
///
/// ```ignore
/// #[derive(AnsiOutput)]
/// #[vtansi(escst, finalbyte = '_')]
/// struct ApplicationProgramCommand {
///     data: String,
/// }
/// // Encodes as: ESC _ data ST
/// ```
pub fn escst(_: EscStArgs) {}

/// Arguments for ESC...ST sequences.
#[allow(dead_code)]
pub struct EscStArgs {
    /// Final byte that follows ESC.
    pub finalbyte: char,

    /// Static data string.
    pub data: &'static str,
}

/// Anchor value for ESC...ST argument field access.
pub const ESCST_ARGS: EscStArgs = EscStArgs {
    finalbyte: '\0',
    data: "",
};

/// SS3 (Single Shift 3) sequence marker.
///
/// Marks a struct as an SS3 sequence (`ESC O`). SS3 sequences are primarily
/// used for parsing keyboard input in application cursor mode.
///
/// **Note:** SS3 is only valid for `AnsiInput`, not `AnsiOutput`.
///
/// # Constraints
///
/// SS3 sequences must either:
/// - Have static `data` attribute with no fields, OR
/// - Have exactly one field (which captures the data byte)
///
/// # Examples
///
/// Static data (unit struct):
/// ```ignore
/// #[derive(AnsiInput)]
/// #[vtansi(ss3, data = "A")]
/// struct SS3CursorUp;
/// // Parses: ESC O A
/// ```
///
/// Dynamic data (single field):
/// ```ignore
/// #[derive(AnsiInput)]
/// #[vtansi(ss3)]
/// struct SS3Key {
///     key: u8,
/// }
/// // Parses: ESC O <key>
/// ```
pub fn ss3(_: Ss3Args) {}

/// Arguments for SS3 sequences.
#[allow(dead_code)]
pub struct Ss3Args {
    /// Static data byte(s) after `ESC O`.
    pub data: &'static str,
}

/// Anchor value for SS3 argument field access.
pub const SS3_ARGS: Ss3Args = Ss3Args { data: "" };

/// C0 control code marker.
///
/// Marks a struct as a C0 control character. C0 codes are single-byte
/// control characters in the range 0x00-0x1F.
///
/// # Required Attributes
///
/// - `code = 0xNN` - The control code byte (must be 0x00-0x1F)
///
/// # Example
///
/// ```ignore
/// #[derive(AnsiOutput)]
/// #[vtansi(c0, code = 0x0E)]
/// struct ShiftOut;
/// // Encodes as: 0x0E (single byte)
/// ```
pub fn c0(_: C0Args) {}

/// Arguments for C0 sequences.
#[allow(dead_code)]
pub struct C0Args {
    /// C0 control code value.
    ///
    /// The byte value of the control code. Must be in range 0x00-0x1F.
    ///
    /// # Syntax
    ///
    /// ```ignore
    /// #[vtansi(c0, code = 0x0E)]  // Shift Out
    /// #[vtansi(c0, code = 0x0F)]  // Shift In
    /// ```
    pub code: u8,
}

/// Anchor value for C0 argument field access.
pub const C0_ARGS: C0Args = C0Args { code: 0 };

/// Single byte code marker (input only).
///
/// Marks a struct as a single byte value. Unlike `c0`, this allows any
/// byte value in the range 0x00-0xFF.
///
/// **Note:** `byte` is only valid for `AnsiInput`, not `AnsiOutput`.
///
/// # Required Attributes
///
/// - `code = 0xNN` - The byte value (0x00-0xFF)
///
/// # Example
///
/// ```ignore
/// #[derive(AnsiInput)]
/// #[vtansi(byte, code = 0xFF)]
/// struct CustomByte;
/// // Parses: 0xFF
/// ```
pub fn byte(_: ByteArgs) {}

/// Arguments for byte sequences.
#[allow(dead_code)]
pub struct ByteArgs {
    /// Byte code value.
    ///
    /// The byte value. Can be any value 0x00-0xFF for `AnsiInput`.
    pub code: u8,
}

/// Anchor value for byte argument field access.
pub const BYTE_ARGS: ByteArgs = ByteArgs { code: 0 };

// =============================================================================
// STANDALONE ATTRIBUTES (used across multiple sequence types or on fields)
// =============================================================================

/// Final byte that terminates a control sequence.
///
/// Required for CSI and DCS sequences. Specifies the character that ends
/// the parameter list and defines the command.
///
/// # Syntax
///
/// Single character:
/// ```ignore
/// #[vtansi(csi, finalbyte = 'H')]
/// ```
///
/// Multiple alternatives (using `|` operator):
/// ```ignore
/// #[vtansi(csi, finalbyte = 'h' | 'l')]
/// ```
///
/// # Valid Range
///
/// Final bytes are typically in the range 0x40-0x7E (`@` through `~`).
pub const finalbyte: () = ();

/// Private marker byte for CSI sequences.
///
/// Specifies a private-use marker that appears after `ESC [` but before
/// parameters. Common values are `?`, `>`, `<`, and `=`.
///
/// # Example
///
/// ```ignore
/// #[derive(AnsiOutput)]
/// #[vtansi(csi, private = '?', finalbyte = 'h')]
/// struct DecSetMode { mode: u16 }
/// // Encodes as: ESC [ ? mode h
/// ```
///
/// # Common Uses
///
/// - `?` - DEC private modes (e.g., `ESC [ ? 1049 h` for alternate screen)
/// - `>` - Secondary device attributes
/// - `=` - Tertiary device attributes
pub const private: () = ();

/// Static parameter values for control sequences.
///
/// Specifies constant parameters for unit structs that have no variable data.
/// Each parameter is a string that will be encoded as-is.
///
/// # Syntax
///
/// ```ignore
/// #[vtansi(csi, params = ["6"], finalbyte = 'n')]
/// struct ReportCursorPosition;
/// // Encodes as: ESC [ 6 n
/// ```
///
/// Multiple parameters:
/// ```ignore
/// #[vtansi(csi, params = ["1", "2"], finalbyte = 'm')]
/// ```
///
/// Alternative parameters (using `|` operator):
/// ```ignore
/// #[vtansi(csi, params = ["12"] | ["13"], finalbyte = 'n')]
/// struct DeviceStatusReport;
/// // Matches both: ESC [ 12 n  and  ESC [ 13 n
/// // Both sequences map to the same handler
/// ```
///
/// This is useful for sequences where multiple static parameter values
/// should be parsed into the same struct type.
pub const params: () = ();

/// Intermediate bytes in control sequences.
///
/// Specifies intermediate bytes that appear between parameters and the
/// final byte. Maximum of 2 intermediate bytes are allowed.
///
/// # Syntax
///
/// Single character:
/// ```ignore
/// #[vtansi(csi, intermediate = ' ', finalbyte = 'q')]
/// ```
///
/// Multiple characters (as string):
/// ```ignore
/// #[vtansi(csi, intermediate = " $", finalbyte = 'p')]
/// ```
pub const intermediate: () = ();

/// Static data string for DCS, OSC, and SS3 sequences.
///
/// Specifies constant data that appears in the sequence. The meaning
/// depends on the sequence type:
///
/// - **DCS**: Data after the final byte, before ST
/// - **OSC**: Data after the number and separator
/// - **SS3**: The data byte(s) after `ESC O`
///
/// # Example
///
/// ```ignore
/// #[derive(AnsiOutput)]
/// #[vtansi(dcs, finalbyte = 'q', data = "$q")]
/// struct RequestStatusString { ... }
/// ```
pub const data: () = ();

/// OSC numeric parameter (Ps).
///
/// Required for OSC sequences. Specifies the numeric command identifier
/// that appears after `ESC ]` and before the semicolon separator.
///
/// # Common OSC Numbers
///
/// - `0` - Set icon name and window title
/// - `1` - Set icon name
/// - `2` - Set window title
/// - `4` - Set/query color palette
/// - `52` - Clipboard operations
/// - `133` - Shell integration (prompt markers)
///
/// # Example
///
/// ```ignore
/// #[derive(AnsiOutput)]
/// #[vtansi(osc, number = "0")]
/// struct SetTitle { title: String }
/// // Encodes as: ESC ] 0 ; title ST
/// ```
pub const number: () = ();

/// Custom separator between static data and parameters.
///
/// Overrides the default `;` separator used between the static data/number
/// and the first dynamic parameter in OSC and DCS sequences.
///
/// # Example
///
/// ```ignore
/// #[vtansi(osc, number = "52", data_delimiter = ";")]
/// ```
pub const data_delimiter: () = ();

/// C0 or byte control code value.
///
/// Required for `c0` and `byte` sequence types. Specifies the actual
/// byte value of the control code.
///
/// # Syntax
///
/// ```ignore
/// #[vtansi(c0, code = 0x0E)]  // Shift Out
/// #[vtansi(c0, code = 0x0F)]  // Shift In
/// ```
///
/// # Valid Ranges
///
/// - **c0**: 0x00-0x1F (C0 control characters)
/// - **byte**: 0x00-0xFF (any byte value, input only)
pub const code: () = ();

// =============================================================================
// FORMAT AND ENCODING ATTRIBUTES
// =============================================================================

/// Struct encoding format.
///
/// Specifies how struct fields are encoded/decoded. Two formats are available:
///
/// # `"map"` (default for named fields)
///
/// Fields are encoded as `key=value` pairs separated by a delimiter:
/// ```ignore
/// #[derive(FromAnsi, ToAnsi)]
/// struct Settings { width: u32, height: u32 }
/// // Encodes as: "width=800;height=600"
/// ```
///
/// # `"vector"` (default for tuple structs)
///
/// Fields are encoded as values only, separated by a delimiter:
/// ```ignore
/// #[derive(FromAnsi, ToAnsi)]
/// #[vtansi(format = "vector")]
/// struct Point { x: i32, y: i32 }
/// // Encodes as: "100;200"
/// ```
///
/// **Note**: Tuple structs automatically use `vector` format and cannot
/// use `map` format (since they have no field names).
pub const format: () = ();

/// Field delimiter for struct encoding.
///
/// Specifies the separator between fields when encoding structs.
/// Default is `;` (semicolon).
///
/// # Syntax
///
/// Byte literal:
/// ```ignore
/// #[vtansi(delimiter = b';')]
/// ```
///
/// Character literal:
/// ```ignore
/// #[vtansi(delimiter = ',')]
/// ```
///
/// # Example
///
/// ```ignore
/// #[derive(FromAnsi, ToAnsi)]
/// #[vtansi(delimiter = ",")]
/// struct Config { name: String, value: u32 }
/// // Encodes as: "name=foo,value=42"
/// ```
pub const delimiter: () = ();

/// Transparent wrapper marker.
///
/// Marks a struct as a transparent wrapper around a single field.
/// Encoding/decoding delegates directly to the inner field.
///
/// # Requirements
///
/// - Struct must have exactly one field
/// - For `Vec<T>` fields, `delimiter` must also be specified
///
/// # Examples
///
/// Simple wrapper:
/// ```ignore
/// #[derive(FromAnsi, ToAnsi)]
/// #[vtansi(transparent)]
/// struct MyInt(u32);
/// ```
///
/// Vec wrapper (requires delimiter):
/// ```ignore
/// #[derive(FromAnsi, ToAnsi)]
/// #[vtansi(transparent, delimiter = b';')]
/// struct Numbers(Vec<u32>);
/// // Encodes: Numbers(vec![1, 2, 3]) -> "1;2;3"
/// ```
pub const transparent: () = ();

/// Constant encoded length for enums.
///
/// Specifies a fixed encoded length for the `ENCODED_LEN` associated constant.
/// Only valid for enums without `#[repr(...)]` attributes.
///
/// # Example
///
/// ```ignore
/// #[derive(ToAnsi)]
/// #[vtansi(encoded_len = 1)]
/// enum SingleCharCommand {
///     Save,
///     Load,
/// }
/// ```
///
/// # Restrictions
///
/// - Cannot be used with `#[repr(u8)]` or similar
/// - Not valid for structs
pub const encoded_len: () = ();

// =============================================================================
// TYPE CONVERSION ATTRIBUTES
// =============================================================================

/// Custom target type for parsing.
///
/// Specifies a custom type to convert into during parsing. The generated
/// `TryFromAnsi` implementation will convert to the specified type.
///
/// # Example
///
/// ```ignore
/// #[derive(FromAnsi)]
/// #[vtansi(into = MyCustomType)]
/// struct ParseHelper { ... }
/// ```
pub const into: Type = Type;

/// Alias for another type (registry).
///
/// Marks this type as an alias for another type. The alias will encode
/// to the same byte sequence but will not register in the parser trie
/// (avoiding duplicate registrations).
///
/// # Example
///
/// ```ignore
/// #[derive(AnsiOutput)]
/// #[vtansi(csi, finalbyte = 'H', alias_of = CursorPosition)]
/// struct CUP {
///     row: u16,
///     col: u16,
/// }
/// ```
pub const alias_of: Type = Type;

/// Use exact param count for trie key disambiguation.
///
/// This attribute is used to disambiguate CSI sequences that share the same
/// final byte but differ in parameter count. When marked with `disambiguate`,
/// the sequence uses `2 + total_params` as the key byte instead of the boolean
/// `has_params` marker.
///
/// # Requirements
///
/// - The sequence must be a CSI sequence
/// - All parameters must be required (no optional params)
///
/// # How It Works
///
/// The parser first tries the normal `has_params` lookup (0 or 1). If that
/// fails for a sequence with 2+ params, it tries `2 + param_count` as a
/// fallback to match disambiguated sequences.
///
/// # Example
///
/// ```ignore
/// // ScrollDown uses final byte 'T' with 0-1 params
/// #[derive(AnsiOutput)]
/// #[vtansi(csi, finalbyte = 'T')]
/// struct ScrollDown(pub u16);
///
/// // TrackMouse also uses final byte 'T' but with 5 params
/// // Using disambiguate allows both to coexist
/// #[derive(AnsiOutput)]
/// #[vtansi(csi, finalbyte = 'T', disambiguate)]
/// struct TrackMouse {
///     cmd: u8,
///     start_column: u16,
///     start_row: u16,
///     first_row: u16,
///     last_row: u16,
/// }
/// ```
pub const disambiguate: () = ();

// =============================================================================
// FIELD LOCATION ATTRIBUTES
// =============================================================================

/// Default field location for all fields.
///
/// Specifies where all fields are located within a control sequence by default.
/// Individual fields can override this with `#[vtansi(locate = ...)]`.
///
/// # Locations
///
/// - `params` - Fields are in the parameter section
/// - `data` - Fields are in the data section
/// - `final` - Field is the final byte
///
/// # Example
///
/// ```ignore
/// #[derive(AnsiOutput)]
/// #[vtansi(csi, finalbyte = 'm', locate_all = params)]
/// struct SetGraphicsRendition { ... }
/// ```
pub const locate_all: () = ();

/// Field location within a control sequence (field attribute).
///
/// Specifies where a specific field is located within the control sequence.
/// Overrides the type-level `locate_all` setting.
///
/// # Locations
///
/// - `params` - Field is in the parameter section
/// - `data` - Field is in the data section
/// - `final` - Field is the final byte
///
/// # Example
///
/// ```ignore
/// #[derive(AnsiOutput)]
/// #[vtansi(dcs, finalbyte = 'q')]
/// struct Query {
///     #[vtansi(locate = params)]
///     param: u32,
///     #[vtansi(locate = data)]
///     data: String,
/// }
/// ```
pub const locate: () = ();

// =============================================================================
// FIELD ATTRIBUTES
// =============================================================================

/// Skip field during encoding/decoding.
///
/// Marks a field to be excluded from ANSI encoding and decoding.
/// Skipped fields must have a default value or be wrapped in `Option`.
///
/// # Example
///
/// ```ignore
/// #[derive(FromAnsi, ToAnsi)]
/// struct Data {
///     id: u32,
///     #[vtansi(skip)]
///     internal: String,  // Not encoded/decoded
/// }
/// ```
pub const skip: () = ();

/// Multiplex field with another parameter position.
///
/// Enables multiplexing where multiple fields can encode to and decode
/// from the same parameter position. Useful for bit-field extraction
/// or modifier flags.
///
/// # Syntax
///
/// ```ignore
/// #[vtansi(muxwith = field_name)]
/// ```
///
/// # Example
///
/// ```ignore
/// #[derive(FromAnsi, ToAnsi)]
/// #[vtansi(format = "vector")]
/// struct MouseEvent {
///     button_code: u8,           // Primary encoding
///     #[vtansi(muxwith = button_code)]
///     modifiers: Modifiers,      // Extracts from same position
/// }
/// ```
///
/// The `muxwith` field's type must implement `AnsiMuxEncode` for encoding
/// and `TryFromAnsi` for decoding.
pub const muxwith: () = ();

/// Flatten nested structure.
///
/// Delegates parameter parsing to the field's `try_from_ansi_iter` method
/// instead of consuming a single parameter. Useful for nested structures
/// that consume multiple parameters.
///
/// # Example
///
/// ```ignore
/// #[derive(FromAnsi)]
/// struct Outer {
///     prefix: u32,
///     #[vtansi(flatten)]
///     inner: Inner,  // Consumes remaining parameters
/// }
/// ```
pub const flatten: () = ();

// =============================================================================
// VARIANT ATTRIBUTES
// =============================================================================

/// Default variant for unrecognized values.
///
/// Marks an enum variant as the fallback for values that don't match
/// any other variant. Only one variant can be marked as default.
///
/// # Unit Default
///
/// Returns a constant value for unrecognized inputs:
/// ```ignore
/// #[derive(FromAnsi)]
/// #[repr(u8)]
/// enum Color {
///     Red = 0,
///     Green = 1,
///     #[vtansi(default)]
///     Unknown = 255,
/// }
/// ```
///
/// # Capturing Default
///
/// Captures the unrecognized value in a tuple variant:
/// ```ignore
/// #[derive(FromAnsi)]
/// #[repr(u8)]
/// enum Status {
///     Ok = 200,
///     NotFound = 404,
///     #[vtansi(default)]
///     Other(u8),  // Captures the actual value
/// }
/// ```
pub const default: () = ();
