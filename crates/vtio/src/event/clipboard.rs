//! Clipboard operations (OSC 52).
//!
//! This module implements the OSC 52 sequence for clipboard manipulation,
//! which is widely used by modern terminal applications (neovim, tmux, etc.)
//! for system clipboard integration.
//!
//! # Sequence Format
//!
//! ```text
//! OSC 52 ; Pc ; Pd ST
//! ```
//!
//! Where:
//! - `Pc` specifies clipboard targets: `c` (clipboard), `p` (primary),
//!   `s` (system default), `q` (secondary), `0-7` (cut buffers)
//! - `Pd` is base64-encoded data, or `?` to query
//!
//! # Security Note
//!
//! This sequence is disabled in many terminal emulator default configurations
//! because it can be dangerous - malicious content could potentially read or
//! write clipboard data without user consent.

use std::fmt;

/// Clipboard target for OSC 52 operations.
///
/// These correspond to X11 selection types (except for `System` which is
/// the terminal's configured default).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClipboardTarget {
    /// System default clipboard (configured by terminal).
    System,
    /// Primary selection (X11) - typically set by selecting text.
    Primary,
    /// Secondary selection (X11) - rarely used.
    Secondary,
    /// Clipboard selection (X11) - typically set by explicit copy.
    Clipboard,
    /// Cut buffer 0-7 (X11 legacy feature).
    CutBuffer(u8),
}

impl ClipboardTarget {
    /// Convert this target to its character representation.
    #[must_use]
    pub const fn as_char(&self) -> char {
        match self {
            Self::System => 's',
            Self::Primary => 'p',
            Self::Secondary => 'q',
            Self::Clipboard => 'c',
            Self::CutBuffer(n) => match n {
                1 => '1',
                2 => '2',
                3 => '3',
                4 => '4',
                5 => '5',
                6 => '6',
                7 => '7',
                _ => '0', // Default to 0 for 0 and invalid values
            },
        }
    }

    /// Try to parse a clipboard target from a character.
    #[must_use]
    pub const fn from_char(c: char) -> Option<Self> {
        match c {
            's' => Some(Self::System),
            'p' => Some(Self::Primary),
            'q' => Some(Self::Secondary),
            'c' => Some(Self::Clipboard),
            '0' => Some(Self::CutBuffer(0)),
            '1' => Some(Self::CutBuffer(1)),
            '2' => Some(Self::CutBuffer(2)),
            '3' => Some(Self::CutBuffer(3)),
            '4' => Some(Self::CutBuffer(4)),
            '5' => Some(Self::CutBuffer(5)),
            '6' => Some(Self::CutBuffer(6)),
            '7' => Some(Self::CutBuffer(7)),
            _ => None,
        }
    }
}

impl fmt::Display for ClipboardTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_char())
    }
}

/// Clipboard action for OSC 52 operations.
///
/// This enum represents either a query (`?`) or data to set (base64-encoded).
/// It is used as the data portion of OSC 52 sequences, similar to how
/// [`crate::event::color::TerminalColorAction`] handles color queries/sets.
///
/// See <https://terminalguide.namepad.de/seq/osc-52/> for format details.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum ClipboardAction<'a> {
    /// Query the current clipboard contents.
    #[default]
    Query,
    /// Set clipboard to the specified base64-encoded data.
    Set(&'a str),
}

impl<'a> ClipboardAction<'a> {
    /// Create a query action.
    #[must_use]
    pub const fn query() -> Self {
        Self::Query
    }

    /// Create a set action with base64-encoded data.
    #[must_use]
    pub const fn set(base64_data: &'a str) -> Self {
        Self::Set(base64_data)
    }

    /// Check if this is a query action.
    #[must_use]
    pub const fn is_query(&self) -> bool {
        matches!(self, Self::Query)
    }

    /// Get the data if this is a Set action.
    #[must_use]
    pub const fn as_set(&self) -> Option<&str> {
        match self {
            Self::Set(data) => Some(data),
            Self::Query => None,
        }
    }

    /// Convert to an owned version.
    #[must_use]
    pub fn to_owned(&self) -> ClipboardActionOwned {
        match self {
            Self::Query => ClipboardActionOwned::Query,
            Self::Set(data) => ClipboardActionOwned::Set((*data).to_string()),
        }
    }
}

impl vtansi::AnsiEncode for ClipboardAction<'_> {
    fn encode_ansi_into<W: std::io::Write + ?Sized>(
        &self,
        sink: &mut W,
    ) -> Result<usize, vtansi::EncodeError> {
        match self {
            Self::Query => '?'.encode_ansi_into(sink),
            Self::Set(data) => data.encode_ansi_into(sink),
        }
    }
}

impl<'a> vtansi::TryFromAnsi<'a> for ClipboardAction<'a> {
    fn try_from_ansi(bytes: &'a [u8]) -> Result<Self, vtansi::ParseError> {
        if bytes == b"?" {
            Ok(Self::Query)
        } else {
            match std::str::from_utf8(bytes) {
                Ok(s) => Ok(Self::Set(s)),
                Err(e) => Err(vtansi::ParseError::InvalidValue(e.to_string())),
            }
        }
    }
}

/// Owned version of [`ClipboardAction`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum ClipboardActionOwned {
    /// Query the current clipboard contents.
    #[default]
    Query,
    /// Set clipboard to the specified base64-encoded data.
    Set(String),
}

impl ClipboardActionOwned {
    /// Borrow this owned action as a borrowed version.
    #[must_use]
    pub fn borrow(&self) -> ClipboardAction<'_> {
        match self {
            Self::Query => ClipboardAction::Query,
            Self::Set(data) => ClipboardAction::Set(data),
        }
    }

    /// Check if this is a query action.
    #[must_use]
    pub const fn is_query(&self) -> bool {
        matches!(self, Self::Query)
    }

    /// Get the data if this is a Set action.
    #[must_use]
    pub fn as_set(&self) -> Option<&str> {
        match self {
            Self::Set(data) => Some(data),
            Self::Query => None,
        }
    }
}

impl vtansi::AnsiEncode for ClipboardActionOwned {
    fn encode_ansi_into<W: std::io::Write + ?Sized>(
        &self,
        sink: &mut W,
    ) -> Result<usize, vtansi::EncodeError> {
        self.borrow().encode_ansi_into(sink)
    }
}

impl<'a> From<ClipboardAction<'a>> for ClipboardActionOwned {
    fn from(value: ClipboardAction<'a>) -> Self {
        value.to_owned()
    }
}

/// Manipulate clipboard contents (set or query).
///
/// *Sequence*: `OSC 52 ; Pc ; Pd ST`
///
/// This struct handles both setting and querying clipboard contents:
/// - To set: use `ClipboardAction::Set(base64_data)`
/// - To query: use `ClipboardAction::Query`
///
/// Multiple targets can be specified to write to multiple clipboards
/// simultaneously. When reading, the first clipboard with data is returned.
///
/// See <https://terminalguide.namepad.de/seq/osc-52/> for
/// terminal support specifics.
///
/// # Examples
///
/// ```ignore
/// use vtio::event::clipboard::{Clipboard, ClipboardAction};
/// use vtansi::AnsiEncode;
///
/// // Set clipboard to "Hello" (base64 encoded as "SGVsbG8=")
/// let set_cmd = Clipboard {
///     targets: "c",
///     action: ClipboardAction::Set("SGVsbG8="),
/// };
/// assert_eq!(set_cmd.to_ansi_string(), "\x1b]52;c;SGVsbG8=\x1b\\");
///
/// // Query clipboard contents
/// let query_cmd = Clipboard {
///     targets: "c",
///     action: ClipboardAction::Query,
/// };
/// assert_eq!(query_cmd.to_ansi_string(), "\x1b]52;c;?\x1b\\");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, vtansi::derive::AnsiOutput)]
#[vtansi(osc, number = "52", data_delimiter = ';')]
pub struct Clipboard<'a> {
    /// Clipboard targets (e.g., "c" for clipboard, "pc" for primary and clipboard).
    pub targets: &'a str,
    /// The action to perform: query or set with base64 data.
    pub action: ClipboardAction<'a>,
}

impl<'a> Clipboard<'a> {
    /// Create a new clipboard command.
    #[must_use]
    pub const fn new(targets: &'a str, action: ClipboardAction<'a>) -> Self {
        Self { targets, action }
    }

    /// Create a command to set clipboard data.
    #[must_use]
    pub const fn set(targets: &'a str, base64_data: &'a str) -> Self {
        Self {
            targets,
            action: ClipboardAction::Set(base64_data),
        }
    }

    /// Create a command to query clipboard contents.
    #[must_use]
    pub const fn query(targets: &'a str) -> Self {
        Self {
            targets,
            action: ClipboardAction::Query,
        }
    }

    /// Create a command to set the system clipboard.
    #[must_use]
    pub const fn set_system(base64_data: &'a str) -> Self {
        Self::set("s", base64_data)
    }

    /// Create a command to query the system clipboard.
    #[must_use]
    pub const fn query_system() -> Self {
        Self::query("s")
    }

    /// Create a command to set the primary selection.
    #[must_use]
    pub const fn set_primary(base64_data: &'a str) -> Self {
        Self::set("p", base64_data)
    }

    /// Create a command to query the primary selection.
    #[must_use]
    pub const fn query_primary() -> Self {
        Self::query("p")
    }

    /// Create a command to set the clipboard selection.
    #[must_use]
    pub const fn set_clipboard(base64_data: &'a str) -> Self {
        Self::set("c", base64_data)
    }

    /// Create a command to query the clipboard selection.
    #[must_use]
    pub const fn query_clipboard() -> Self {
        Self::query("c")
    }

    /// Convert to an owned version.
    #[must_use]
    pub fn to_owned(&self) -> ClipboardOwned {
        ClipboardOwned {
            targets: self.targets.to_string(),
            action: self.action.to_owned(),
        }
    }
}

/// Owned version of [`Clipboard`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClipboardOwned {
    /// Clipboard targets.
    pub targets: String,
    /// The action to perform.
    pub action: ClipboardActionOwned,
}

impl ClipboardOwned {
    /// Create a new owned clipboard command.
    #[must_use]
    pub fn new(
        targets: impl Into<String>,
        action: ClipboardActionOwned,
    ) -> Self {
        Self {
            targets: targets.into(),
            action,
        }
    }

    /// Borrow this owned struct as a borrowed version.
    #[must_use]
    pub fn borrow(&self) -> Clipboard<'_> {
        Clipboard {
            targets: &self.targets,
            action: self.action.borrow(),
        }
    }
}

impl<'a> From<Clipboard<'a>> for ClipboardOwned {
    fn from(value: Clipboard<'a>) -> Self {
        value.to_owned()
    }
}

impl vtansi::AnsiEncode for ClipboardOwned {
    fn encode_ansi_into<W: std::io::Write + ?Sized>(
        &self,
        sink: &mut W,
    ) -> Result<usize, vtansi::EncodeError> {
        self.borrow().encode_ansi_into(sink)
    }
}

/// Clipboard contents response.
///
/// *Sequence*: `OSC 52 ; Pc ; Pd ST`
///
/// This is the response sent by the terminal when clipboard contents are
/// queried. The format is identical to the set command, containing the
/// target(s) and base64-encoded data.
///
/// # Example
///
/// ```ignore
/// use vtio::event::clipboard::ClipboardResponse;
/// use vtansi::TryFromAnsi;
///
/// // Parse a response containing "Hello" (base64: "SGVsbG8=")
/// let response = ClipboardResponse::try_from_ansi(b"c;SGVsbG8=")?;
/// assert_eq!(response.targets, "c");
/// assert_eq!(response.data, "SGVsbG8=");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, vtansi::derive::AnsiInput)]
#[vtansi(osc, number = "52", data_delimiter = ';')]
pub struct ClipboardResponse<'a> {
    /// Clipboard target(s) the data came from.
    /// Unsupported targets from the query may be removed.
    pub targets: &'a str,
    /// Base64-encoded clipboard data.
    pub data: &'a str,
}

impl ClipboardResponse<'_> {
    /// Convert to an owned version.
    #[must_use]
    pub fn to_owned(&self) -> ClipboardResponseOwned {
        ClipboardResponseOwned {
            targets: self.targets.to_string(),
            data: self.data.to_string(),
        }
    }

    /// Parse the clipboard targets into a vector of [`ClipboardTarget`] values.
    ///
    /// Unknown target characters are silently ignored.
    #[must_use]
    pub fn parse_targets(&self) -> Vec<ClipboardTarget> {
        self.targets
            .chars()
            .filter_map(ClipboardTarget::from_char)
            .collect()
    }
}

/// Owned version of [`ClipboardResponse`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClipboardResponseOwned {
    /// Clipboard target(s) the data came from.
    pub targets: String,
    /// Base64-encoded clipboard data.
    pub data: String,
}

impl ClipboardResponseOwned {
    /// Borrow this owned struct as a borrowed version.
    #[must_use]
    pub fn borrow(&self) -> ClipboardResponse<'_> {
        ClipboardResponse {
            targets: &self.targets,
            data: &self.data,
        }
    }

    /// Parse the clipboard targets into a vector of [`ClipboardTarget`] values.
    ///
    /// Unknown target characters are silently ignored.
    #[must_use]
    pub fn parse_targets(&self) -> Vec<ClipboardTarget> {
        self.borrow().parse_targets()
    }
}

impl<'a> From<ClipboardResponse<'a>> for ClipboardResponseOwned {
    fn from(value: ClipboardResponse<'a>) -> Self {
        value.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vtansi::AnsiEncode;

    #[test]
    fn test_clipboard_target_as_char() {
        assert_eq!(ClipboardTarget::System.as_char(), 's');
        assert_eq!(ClipboardTarget::Primary.as_char(), 'p');
        assert_eq!(ClipboardTarget::Secondary.as_char(), 'q');
        assert_eq!(ClipboardTarget::Clipboard.as_char(), 'c');
        assert_eq!(ClipboardTarget::CutBuffer(0).as_char(), '0');
        assert_eq!(ClipboardTarget::CutBuffer(7).as_char(), '7');
    }

    #[test]
    fn test_clipboard_target_from_char() {
        assert_eq!(
            ClipboardTarget::from_char('s'),
            Some(ClipboardTarget::System)
        );
        assert_eq!(
            ClipboardTarget::from_char('p'),
            Some(ClipboardTarget::Primary)
        );
        assert_eq!(
            ClipboardTarget::from_char('q'),
            Some(ClipboardTarget::Secondary)
        );
        assert_eq!(
            ClipboardTarget::from_char('c'),
            Some(ClipboardTarget::Clipboard)
        );
        assert_eq!(
            ClipboardTarget::from_char('0'),
            Some(ClipboardTarget::CutBuffer(0))
        );
        assert_eq!(
            ClipboardTarget::from_char('7'),
            Some(ClipboardTarget::CutBuffer(7))
        );
        assert_eq!(ClipboardTarget::from_char('x'), None);
    }

    #[test]
    fn test_clipboard_action_query_encoding() {
        let action = ClipboardAction::Query;
        let mut buf = Vec::new();
        action.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "?");
    }

    #[test]
    fn test_clipboard_action_set_encoding() {
        let action = ClipboardAction::Set("SGVsbG8=");
        let mut buf = Vec::new();
        action.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "SGVsbG8=");
    }

    #[test]
    fn test_clipboard_set_encoding() {
        let cmd = Clipboard::set("c", "SGVsbG8=");
        let mut buf = Vec::new();
        cmd.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "\x1b]52;c;SGVsbG8=\x1b\\");
    }

    #[test]
    fn test_clipboard_query_encoding() {
        let cmd = Clipboard::query("c");
        let mut buf = Vec::new();
        cmd.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "\x1b]52;c;?\x1b\\");
    }

    #[test]
    fn test_clipboard_set_multiple_targets() {
        let cmd = Clipboard::set("pc", "SGVsbG8=");
        let mut buf = Vec::new();
        cmd.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(
            String::from_utf8(buf).unwrap(),
            "\x1b]52;pc;SGVsbG8=\x1b\\"
        );
    }

    #[test]
    fn test_clipboard_query_multiple_targets() {
        let cmd = Clipboard::query("pc");
        let mut buf = Vec::new();
        cmd.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "\x1b]52;pc;?\x1b\\");
    }

    #[test]
    fn test_clipboard_set_system() {
        let cmd = Clipboard::set_system("dGVzdA==");
        let mut buf = Vec::new();
        cmd.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "\x1b]52;s;dGVzdA==\x1b\\");
    }

    #[test]
    fn test_clipboard_query_system() {
        let cmd = Clipboard::query_system();
        let mut buf = Vec::new();
        cmd.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "\x1b]52;s;?\x1b\\");
    }

    #[test]
    fn test_clipboard_owned_set() {
        let cmd = ClipboardOwned::new(
            "c",
            ClipboardActionOwned::Set("SGVsbG8=".to_string()),
        );
        let mut buf = Vec::new();
        cmd.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "\x1b]52;c;SGVsbG8=\x1b\\");
    }

    #[test]
    fn test_clipboard_owned_query() {
        let cmd = ClipboardOwned::new("c", ClipboardActionOwned::Query);
        let mut buf = Vec::new();
        cmd.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "\x1b]52;c;?\x1b\\");
    }

    #[test]
    fn test_clipboard_response_parse_targets() {
        let response = ClipboardResponse {
            targets: "pc",
            data: "SGVsbG8=",
        };
        let targets = response.parse_targets();
        assert_eq!(
            targets,
            vec![ClipboardTarget::Primary, ClipboardTarget::Clipboard]
        );
    }

    #[test]
    fn test_clipboard_response_to_owned() {
        let response = ClipboardResponse {
            targets: "c",
            data: "SGVsbG8=",
        };
        let owned = response.to_owned();
        assert_eq!(owned.targets, "c");
        assert_eq!(owned.data, "SGVsbG8=");
    }

    #[test]
    fn test_clipboard_to_owned() {
        let cmd = Clipboard::set("c", "SGVsbG8=");
        let owned = cmd.to_owned();
        assert_eq!(owned.targets, "c");
        assert!(
            matches!(owned.action, ClipboardActionOwned::Set(ref s) if s == "SGVsbG8=")
        );

        // Verify the owned version encodes the same
        let mut buf1 = Vec::new();
        let mut buf2 = Vec::new();
        cmd.encode_ansi_into(&mut buf1).unwrap();
        owned.encode_ansi_into(&mut buf2).unwrap();
        assert_eq!(buf1, buf2);
    }

    #[test]
    fn test_clipboard_query_to_owned() {
        let cmd = Clipboard::query("pc");
        let owned = cmd.to_owned();
        assert_eq!(owned.targets, "pc");
        assert!(matches!(owned.action, ClipboardActionOwned::Query));

        // Verify the owned version encodes the same
        let mut buf1 = Vec::new();
        let mut buf2 = Vec::new();
        cmd.encode_ansi_into(&mut buf1).unwrap();
        owned.encode_ansi_into(&mut buf2).unwrap();
        assert_eq!(buf1, buf2);
    }

    #[test]
    fn test_clipboard_action_is_query() {
        assert!(ClipboardAction::Query.is_query());
        assert!(!ClipboardAction::Set("data").is_query());
        assert!(ClipboardActionOwned::Query.is_query());
        assert!(!ClipboardActionOwned::Set("data".to_string()).is_query());
    }

    #[test]
    fn test_clipboard_action_as_set() {
        assert_eq!(ClipboardAction::Query.as_set(), None);
        assert_eq!(ClipboardAction::Set("data").as_set(), Some("data"));
        assert_eq!(ClipboardActionOwned::Query.as_set(), None);
        assert_eq!(
            ClipboardActionOwned::Set("data".to_string()).as_set(),
            Some("data")
        );
    }
}
