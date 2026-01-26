//! iTerm2 proprietary escape sequences (OSC 1337).
//!
//! iTerm2 defines a set of proprietary escape sequences under OSC 1337
//! for terminal control and communication. These sequences follow the
//! pattern:
//!
//! ```text
//! ESC ] 1337 ; [command] ST
//! ```
//!
//! Where `[command]` can be:
//! - A simple key (e.g., `SetMark`)
//! - A key=value pair (e.g., `CursorShape=1`)
//! - Multiple key=value pairs separated by semicolons (e.g.,
//!   `Block=id=foo;attr=start`)
//!
//! This module provides type-safe wrappers for known sequences and a
//! generic mechanism for encoding arbitrary key=value pairs.

use vtansi::derive::{FromAnsi, ToAnsi};

/// Set a mark at the current cursor position.
///
/// *Sequence*: `OSC 1337 ; SetMark ST`
///
/// Equivalent to the "Set Mark" command (cmd-shift-M).
/// The mark can be jumped to later with cmd-shift-J.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, vtansi::derive::AnsiOutput,
)]
#[vtansi(osc, number = "1337", data = "SetMark")]
pub struct SetMark;

/// Bring iTerm2 window to the foreground.
///
/// *Sequence*: `OSC 1337 ; StealFocus ST`
///
/// Force the terminal to steal focus from other applications.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, vtansi::derive::AnsiOutput,
)]
#[vtansi(osc, number = "1337", data = "StealFocus")]
pub struct StealFocus;

/// Clear the scrollback history.
///
/// *Sequence*: `OSC 1337 ; ClearScrollback ST`
///
/// Erase all content in the scrollback buffer, keeping only the
/// visible screen content.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, vtansi::derive::AnsiOutput,
)]
#[vtansi(osc, number = "1337", data = "ClearScrollback")]
pub struct ClearScrollback;

/// End a copy-to-clipboard operation.
///
/// *Sequence*: `OSC 1337 ; EndCopy ST`
///
/// Marks the end of text being copied to the pasteboard. Must be
/// preceded by a `CopyToClipboard` command.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, vtansi::derive::AnsiOutput,
)]
#[vtansi(osc, number = "1337", data = "EndCopy")]
pub struct EndCopy;

/// Report the cell size in points.
///
/// *Sequence*: `OSC 1337 ; ReportCellSize ST`
///
/// The terminal responds with:
/// `OSC 1337 ; ReportCellSize=[height];[width];[scale] ST`
///
/// where scale is the pixel-to-point ratio (1.0 for non-retina,
/// 2.0 for retina).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, vtansi::derive::AnsiOutput,
)]
#[vtansi(osc, number = "1337", data = "ReportCellSize")]
pub struct ReportCellSize;

/// Push the current touch bar key labels onto a stack.
///
/// *Sequence*: `OSC 1337 ; PushKeyLabels ST`
///
/// Save the current set of function key labels for later restoration
/// with `PopKeyLabels`.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, vtansi::derive::AnsiOutput,
)]
#[vtansi(osc, number = "1337", data = "PushKeyLabels")]
pub struct PushKeyLabels;

/// Pop touch bar key labels from the stack.
///
/// *Sequence*: `OSC 1337 ; PopKeyLabels ST`
///
/// Restore the most recently pushed set of function key labels.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, vtansi::derive::AnsiOutput,
)]
#[vtansi(osc, number = "1337", data = "PopKeyLabels")]
pub struct PopKeyLabels;

/// Disinter a buried session.
///
/// *Sequence*: `OSC 1337 ; Disinter ST`
///
/// Restore a previously buried session to the active state.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, vtansi::derive::AnsiOutput,
)]
#[vtansi(osc, number = "1337", data = "Disinter")]
pub struct Disinter;

/// Clear captured output.
///
/// *Sequence*: `OSC 1337 ; ClearCapturedOutput ST`
///
/// Erase the current captured output buffer for this session.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, vtansi::derive::AnsiOutput,
)]
#[vtansi(osc, number = "1337", data = "ClearCapturedOutput")]
pub struct ClearCapturedOutput;

// Single parameter commands

/// Cursor shape values.
#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    num_enum::TryFromPrimitive,
    num_enum::IntoPrimitive,
    vtansi::derive::ToAnsi,
    vtansi::derive::FromAnsi,
)]
#[repr(u8)]
pub enum CursorShapeValue {
    /// Block cursor.
    #[default]
    Block = 0,
    /// Vertical bar cursor.
    VerticalBar = 1,
    /// Underline cursor.
    Underline = 2,
}

/// Set the cursor shape.
///
/// *Sequence*: `OSC 1337 ; CursorShape = Ps ST`
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, vtansi::derive::AnsiOutput,
)]
#[vtansi(osc, number = "1337", data = "CursorShape", data_delimiter = '=')]
pub struct CursorShape {
    pub shape: CursorShapeValue,
}

/// Set the current directory path.
///
/// *Sequence*: `OSC 1337 ; CurrentDir = Pt ST`
///
/// Inform iTerm2 of the current working directory to enable
/// semantic history and other path-based features.
#[derive(Debug, Clone, PartialEq, Eq, Hash, vtansi::derive::AnsiOutput)]
#[vtansi(osc, number = "1337", data = "CurrentDir", data_delimiter = '=')]
pub struct CurrentDir<'a> {
    pub path: &'a str,
}

/// Change the session's profile.
///
/// *Sequence*: `OSC 1337 ; SetProfile = Pt ST`
///
/// Switch to a different profile by name. The profile must exist
/// in iTerm2's configuration.
#[derive(Debug, Clone, PartialEq, Eq, Hash, vtansi::derive::AnsiOutput)]
#[vtansi(osc, number = "1337", data = "SetProfile", data_delimiter = '=')]
pub struct SetProfile<'a> {
    pub profile: &'a str,
}

/// Clipboard selector.
#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    strum::IntoStaticStr,
    strum::Display,
    strum::EnumString,
    ToAnsi,
    FromAnsi,
)]
#[strum(serialize_all = "lowercase")]
pub enum Clipboard {
    /// General clipboard
    #[strum(serialize = "")]
    #[default]
    General,
    /// Rule clipboard
    Rule,
    /// Find clipboard
    Find,
    /// Font clipboard
    Font,
}

/// Start copying text to a clipboard.
///
/// *Sequence*: `OSC 1337 ; CopyToClipboard = Ps ST`
///
/// All text received after this command is placed in the specified
/// pasteboard until `EndCopy` is received.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, vtansi::derive::AnsiOutput,
)]
#[vtansi(osc, number = "1337", data = "CopyToClipboard", data_delimiter = '=')]
pub struct CopyToClipboard {
    pub clipboard: Clipboard,
}

/// Set background image from a file path.
///
/// *Sequence*: `OSC 1337 ; SetBackgroundImageFile = Pt ST`
///
/// The value should be a base64-encoded filename. An empty string
/// removes the background image. User confirmation is required as
/// a security measure.
#[derive(Debug, Clone, PartialEq, Eq, Hash, vtansi::derive::AnsiOutput)]
#[vtansi(
    osc,
    number = "1337",
    data = "SetBackgroundImageFile",
    data_delimiter = '='
)]
pub struct SetBackgroundImageFile<'a> {
    pub base64_path: &'a str,
}

/// Attention request modes.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    strum::IntoStaticStr,
    strum::Display,
    strum::EnumString,
    vtansi::derive::ToAnsi,
    vtansi::derive::FromAnsi,
)]
#[strum(serialize_all = "lowercase")]
pub enum AttentionMode {
    /// Bounce dock icon indefinitely.
    Yes,
    /// Bounce dock icon once.
    Once,
    /// Cancel previous request.
    No,
    /// Display fireworks at cursor location.
    Fireworks,
}

/// Request attention with visual effects.
///
/// *Sequence*: `OSC 1337 ; RequestAttention = Ps ST`
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, vtansi::derive::AnsiOutput,
)]
#[vtansi(osc, number = "1337", data = "RequestAttention", data_delimiter = '=')]
pub struct RequestAttention {
    pub mode: AttentionMode,
}

/// Unicode version values.
#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    num_enum::TryFromPrimitive,
    num_enum::IntoPrimitive,
    vtansi::derive::ToAnsi,
    vtansi::derive::FromAnsi,
)]
#[repr(u8)]
pub enum UnicodeVersionValue {
    /// Unicode 8 width tables.
    V8 = 8,
    /// Unicode 9 width tables.
    #[default]
    V9 = 9,
}

/// Set Unicode width table version.
///
/// *Sequence*: `OSC 1337 ; UnicodeVersion = Ps ST`
///
/// Can also push/pop values on a stack using special string values
/// (use `GenericCommand` for that).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, vtansi::derive::AnsiOutput,
)]
#[vtansi(osc, number = "1337", data = "UnicodeVersion", data_delimiter = '=')]
pub struct UnicodeVersion {
    pub version: UnicodeVersionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Iterm2Bool(bool);

impl From<bool> for Iterm2Bool {
    fn from(value: bool) -> Self {
        Self(value)
    }
}

impl vtansi::AnsiEncode for Iterm2Bool {
    const ENCODED_LEN: Option<usize> = Some(3);

    fn encode_ansi_into<W: std::io::Write + ?Sized>(
        &self,
        sink: &mut W,
    ) -> Result<usize, vtansi::EncodeError> {
        if self.0 {
            vtansi::write_bytes_into(sink, b"yes")
        } else {
            vtansi::write_bytes_into(sink, b"no")
        }
    }
}

impl<'a> vtansi::TryFromAnsi<'a> for Iterm2Bool {
    fn try_from_ansi(bytes: &'a [u8]) -> Result<Self, vtansi::ParseError> {
        Ok(match bytes {
            b"yes" => Self(true),
            _ => Self(false),
        })
    }
}

/// Enable or disable the cursor guide (highlight cursor line).
///
/// *Sequence*: `OSC 1337 ; HighlightCursorLine = Ps ST`
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, vtansi::derive::AnsiOutput,
)]
#[vtansi(
    osc,
    number = "1337",
    data = "HighlightCursorLine",
    data_delimiter = '='
)]
pub struct HighlightCursorLine {
    pub enabled: Iterm2Bool,
}

/// Copy text to the general clipboard.
///
/// *Sequence*: `OSC 1337 ; Copy = Pt ST`
#[derive(Debug, Clone, PartialEq, Eq, Hash, vtansi::derive::AnsiOutput)]
#[vtansi(osc, number = "1337", data = "Copy", data_delimiter = '=')]
pub struct Copy<'a> {
    pub base64_text: &'a str,
}

/// Report the value of a session variable.
///
/// *Sequence*: `OSC 1337 ; ReportVariable = Pt ST`
#[derive(Debug, Clone, PartialEq, Eq, Hash, vtansi::derive::AnsiOutput)]
#[vtansi(osc, number = "1337", data = "ReportVariable", data_delimiter = '=')]
pub struct ReportVariable<'a> {
    pub base64_name: &'a str,
}

/// Request file upload from the user.
///
/// *Sequence*: `OSC 1337 ; RequestUpload = Pt ST`
#[derive(Debug, Clone, PartialEq, Eq, Hash, vtansi::derive::AnsiOutput)]
#[vtansi(osc, number = "1337", data = "RequestUpload", data_delimiter = '=')]
pub struct RequestUpload<'a> {
    pub format: &'a str,
}

/// Open a URL in the default browser.
///
/// *Sequence*: `OSC 1337 ; OpenUrl = Pt ST`
#[derive(Debug, Clone, PartialEq, Eq, Hash, vtansi::derive::AnsiOutput)]
#[vtansi(osc, number = "1337", data = "OpenUrl", data_delimiter = '=')]
pub struct OpenUrl<'a> {
    pub base64_url: &'a str,
}

/// A key or key=value pair for generic iTerm2 commands.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum KeyValue<'a> {
    /// A key without a value.
    Key(&'a [u8]),
    /// A key=value pair.
    KeyValue(&'a [u8], &'a [u8]),
}

/// A list of key=value pairs for iTerm2 commands.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyValueList<'a>(Vec<KeyValue<'a>>);

impl KeyValueList<'_> {
    /// Create a new empty list.
    #[must_use]
    pub fn new() -> Self {
        Self(Vec::new())
    }
}

impl Default for KeyValueList<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> vtansi::TryFromAnsi<'a> for KeyValueList<'a> {
    fn try_from_ansi(bytes: &'a [u8]) -> Result<Self, vtansi::ParseError> {
        let vals: Vec<KeyValue> = bytes
            .split(|&b| b == b';')
            .map(|pair| {
                let mut parts = pair.splitn(2, |&b| b == b'=');
                let key_or_val = parts.next().ok_or_else(|| {
                    vtansi::ParseError::InvalidValue(format!(
                        "invalid key=value pair: {pair:?}"
                    ))
                })?;
                if let Some(val) = parts.next() {
                    Ok(KeyValue::KeyValue(key_or_val, val))
                } else {
                    Ok(KeyValue::Key(key_or_val))
                }
            })
            .collect::<Result<_, _>>()?;

        Ok(Self(vals))
    }
}

impl vtansi::AnsiEncode for KeyValueList<'_> {
    fn encode_ansi_into<W: std::io::Write + ?Sized>(
        &self,
        sink: &mut W,
    ) -> Result<usize, vtansi::EncodeError> {
        let mut counter = 0usize;

        for (i, v) in self.0.iter().enumerate() {
            if i > 0 {
                counter +=
                    <_ as vtansi::AnsiEncode>::encode_ansi_into(&b';', sink)?;
            }
            match *v {
                KeyValue::Key(k) => {
                    counter +=
                        <_ as vtansi::AnsiEncode>::encode_ansi_into(k, sink)?;
                }
                KeyValue::KeyValue(k, v) => {
                    counter +=
                        <_ as vtansi::AnsiEncode>::encode_ansi_into(k, sink)?;
                    counter += <_ as vtansi::AnsiEncode>::encode_ansi_into(
                        &b'=', sink,
                    )?;
                    counter +=
                        <_ as vtansi::AnsiEncode>::encode_ansi_into(v, sink)?;
                }
            }
        }

        Ok(counter)
    }
}

/// Generic command for arbitrary key=value pairs.
///
/// *Sequence*: `OSC 1337 ; key=value ; ... ST`
///
/// Use this for unrecognized or custom iTerm2 commands that follow
/// the key=value pattern. Multiple pairs can be separated by semicolons.
#[derive(Debug, Clone, PartialEq, Eq, Hash, vtansi::derive::AnsiOutput)]
#[vtansi(osc, number = "1337")]
pub struct GenericCommand<'a> {
    pub pairs: KeyValueList<'a>,
}

/// Annotation message with optional length parameter.
///
/// The length specifies how many cells the annotation spans.
/// In the wire format, if length is present, it appears BEFORE the message.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AnnotationMessage<'a> {
    pub length: Option<u32>,
    pub message: &'a str,
}

impl<'a> AnnotationMessage<'a> {
    /// Create a new annotation message without length.
    #[must_use]
    pub fn new(message: &'a str) -> Self {
        Self {
            message,
            length: None,
        }
    }

    /// Create a new annotation message with length.
    #[must_use]
    pub fn with_length(message: &'a str, length: u32) -> Self {
        Self {
            message,
            length: Some(length),
        }
    }
}

impl vtansi::AnsiEncode for AnnotationMessage<'_> {
    fn encode_ansi_into<W: std::io::Write + ?Sized>(
        &self,
        sink: &mut W,
    ) -> Result<usize, vtansi::EncodeError> {
        let mut counter = 0usize;

        if let Some(len) = &self.length {
            counter += <_ as vtansi::AnsiEncode>::encode_ansi_into(len, sink)?;
            counter +=
                <_ as vtansi::AnsiEncode>::encode_ansi_into(&b'|', sink)?;
        }

        counter +=
            <_ as vtansi::AnsiEncode>::encode_ansi_into(&self.message, sink)?;

        Ok(counter)
    }
}

impl<'a> vtansi::TryFromAnsi<'a> for AnnotationMessage<'a> {
    fn try_from_ansi(bytes: &'a [u8]) -> Result<Self, vtansi::ParseError> {
        let mut split = bytes.splitn(2, |&b| b == b'|');
        // Iterator guaranteed to return at least one element.
        let len_or_msg = split.next().unwrap();
        if let Some(msg) = split.next() {
            let length: Option<u32> =
                Some(<_ as vtansi::TryFromAnsi>::try_from_ansi(len_or_msg)?);
            let message: &str = <_ as vtansi::TryFromAnsi>::try_from_ansi(msg)?;

            Ok(Self { length, message })
        } else {
            let message: &str =
                <_ as vtansi::TryFromAnsi>::try_from_ansi(len_or_msg)?;

            Ok(Self {
                length: None,
                message,
            })
        }
    }
}

/// Annotation coordinates (x, y position).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ToAnsi, FromAnsi)]
#[vtansi(delimiter = b'|')]
pub struct AnnotationCoords {
    pub x: u32,
    pub y: u32,
}

impl AnnotationCoords {
    /// Create new annotation coordinates.
    #[must_use]
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }
}

/// Add an annotation at the current cursor position.
///
/// *Sequence*: `OSC 1337 ; AddAnnotation = [length|]message[|x|y] ST`
///
/// Annotations appear as clickable markers in the terminal that can have
/// associated text messages and optional length/position parameters.
/// The length and coordinates are optional.
#[derive(Debug, Clone, PartialEq, Eq, Hash, vtansi::derive::AnsiOutput)]
#[vtansi(
    osc,
    number = "1337",
    data = "AddAnnotation",
    data_delimiter = '=',
    delimiter = b'|'
)]
pub struct AddAnnotation<'a> {
    pub message: AnnotationMessage<'a>,
    pub coords: Option<AnnotationCoords>,
}

/// Add a hidden annotation at the current cursor position.
///
/// *Sequence*: `OSC 1337 ; AddHiddenAnnotation = [length|]message[|x|y] ST`
///
/// Similar to `AddAnnotation`, but the annotation is not visible in the
/// terminal UI by default. The length and coordinates are optional.
#[derive(Debug, Clone, PartialEq, Eq, Hash, vtansi::derive::AnsiOutput)]
#[vtansi(
    osc,
    number = "1337",
    data = "AddHiddenAnnotation",
    data_delimiter = '=',
    delimiter = b'|'
)]
pub struct AddHiddenAnnotation<'a> {
    pub message: AnnotationMessage<'a>,
    pub coords: Option<AnnotationCoords>,
}
