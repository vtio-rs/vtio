//! Terminal color palette querying and manipulation.

use std::fmt;
use std::ops::Deref;

use xparsecolor::XColor;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TerminalPaletteColor(pub XColor);

impl TerminalPaletteColor {
    #[must_use]
    pub fn new(color: &XColor) -> Self {
        Self(*color)
    }
}

impl From<XColor> for TerminalPaletteColor {
    fn from(value: XColor) -> Self {
        Self(value)
    }
}

impl From<&XColor> for TerminalPaletteColor {
    fn from(value: &XColor) -> Self {
        Self(*value)
    }
}

impl Deref for TerminalPaletteColor {
    type Target = XColor;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for TerminalPaletteColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <XColor as fmt::Display>::fmt(&self.0, f)
    }
}

impl vtansi::AnsiEncode for TerminalPaletteColor {
    fn encode_ansi_into<W: std::io::Write + ?Sized>(
        &self,
        sink: &mut W,
    ) -> Result<usize, vtansi::EncodeError> {
        self.0.encode_into(sink).map_err(|err| match err {
            xparsecolor::EncodeError::IoError(e) => {
                vtansi::EncodeError::IOError(e)
            }
            xparsecolor::EncodeError::BufferOverflow(sz) => {
                vtansi::EncodeError::BufferOverflow(sz)
            }
        })
    }
}

impl<'a> vtansi::TryFromAnsi<'a> for TerminalPaletteColor {
    fn try_from_ansi(bytes: &'a [u8]) -> Result<Self, vtansi::ParseError> {
        match XColor::try_from_bytes(bytes) {
            Ok(color) => Ok(color.into()),
            Err(err) => Err(vtansi::ParseError::InvalidValue(err.to_string())),
        }
    }
}

/// Color specification for color-related OSC sequences.
///
/// This enum represents either a query (`?`) or a color value in the
/// `XParseColor` `rgb:r/g/b` format. It is used as the data portion
/// of the OSC sequences.
///
/// See <https://terminalguide.namepad.de/seq/osc-4/> for format details.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum TerminalColorAction {
    /// Query the current color value.
    #[default]
    Query,
    /// Set a specific color.
    Set(XColor),
}

impl TerminalColorAction {
    /// Create a query specification.
    #[must_use]
    pub fn query() -> Self {
        Self::Query
    }

    /// Create a color specification from an RGB color.
    #[must_use]
    pub fn set(color: impl Into<XColor>) -> Self {
        Self::Set(color.into())
    }

    /// Check if this is a query.
    #[must_use]
    pub const fn is_query(&self) -> bool {
        matches!(self, Self::Query)
    }

    /// Get the color if this is a Set action.
    #[must_use]
    pub const fn as_set(&self) -> Option<&XColor> {
        match self {
            Self::Set(color) => Some(color),
            Self::Query => None,
        }
    }
}

impl From<XColor> for TerminalColorAction {
    fn from(color: XColor) -> Self {
        Self::Set(color)
    }
}

impl From<&XColor> for TerminalColorAction {
    fn from(color: &XColor) -> Self {
        Self::Set(*color)
    }
}

impl vtansi::AnsiEncode for TerminalColorAction {
    fn encode_ansi_into<W: std::io::Write + ?Sized>(
        &self,
        sink: &mut W,
    ) -> Result<usize, vtansi::EncodeError> {
        match self {
            Self::Query => '?'.encode_ansi_into(sink),
            Self::Set(color) => {
                TerminalPaletteColor::new(color).encode_ansi_into(sink)
            }
        }
    }
}

impl<'a> vtansi::TryFromAnsi<'a> for TerminalColorAction {
    fn try_from_ansi(bytes: &'a [u8]) -> Result<Self, vtansi::ParseError> {
        if bytes == b"?" {
            Ok(Self::Query)
        } else {
            match XColor::try_from_bytes(bytes) {
                Ok(color) => Ok(Self::Set(color)),
                Err(err) => {
                    Err(vtansi::ParseError::InvalidValue(err.to_string()))
                }
            }
        }
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    vtansi::derive::FromAnsi,
    vtansi::derive::ToAnsi,
)]
#[vtansi(format = "vector")]
pub struct TerminalPaletteAction {
    /// The palette index.
    ///
    /// Standard palette indices are 0-255. iTerm2 extends this with:
    /// - `-1` for default foreground color
    /// - `-2` for default background color
    pub index: i16,
    /// The color specification (query or value).
    pub action: TerminalColorAction,
}

impl TerminalPaletteAction {
    /// Create an OSC color event.
    #[must_use]
    pub const fn new(index: i16, spec: TerminalColorAction) -> Self {
        Self {
            index,
            action: spec,
        }
    }

    /// Create a query for a specific palette color index.
    #[must_use]
    pub const fn query(index: i16) -> Self {
        Self {
            index,
            action: TerminalColorAction::Query,
        }
    }

    /// Create a query for the default foreground color (index -1).
    ///
    /// This is an iTerm2 extension.
    #[must_use]
    pub const fn query_foreground() -> Self {
        Self::query(-1)
    }

    /// Create a query for the default background color (index -2).
    ///
    /// This is an iTerm2 extension.
    #[must_use]
    pub const fn query_background() -> Self {
        Self::query(-2)
    }

    /// Create a command to set a specific palette color.
    #[must_use]
    pub const fn set(index: i16, color: XColor) -> Self {
        Self {
            index,
            action: TerminalColorAction::Set(color),
        }
    }

    /// Check if this is a query.
    #[must_use]
    pub const fn is_query(&self) -> bool {
        matches!(self.action, TerminalColorAction::Query)
    }

    /// Check if this is a foreground color (index -1).
    #[must_use]
    pub const fn is_foreground(&self) -> bool {
        self.index == -1
    }

    /// Check if this is a background color (index -2).
    #[must_use]
    pub const fn is_background(&self) -> bool {
        self.index == -2
    }

    /// Check if this is a standard palette color (index 0-255).
    #[must_use]
    pub const fn is_palette(&self) -> bool {
        self.index >= 0 && self.index <= 255
    }
}

/// *Sequence*: `OSC 4 ; Pc ; Pt ST`
#[derive(Debug, Clone, Copy, PartialEq, vtansi::derive::AnsiInput)]
#[vtansi(osc, number = "4")]
pub struct TerminalPaletteColorResponse {
    /// The palette index.
    pub index: i16,
    /// The color specification.
    pub color: TerminalPaletteColor,
}

impl TerminalPaletteColorResponse {
    /// Create an OSC color response event.
    #[must_use]
    pub const fn new(index: i16, color: &TerminalPaletteColor) -> Self {
        Self {
            index,
            color: *color,
        }
    }

    #[must_use]
    pub const fn foreground(color: &TerminalPaletteColor) -> Self {
        Self::new(-1, color)
    }

    #[must_use]
    pub const fn background(color: &TerminalPaletteColor) -> Self {
        Self::new(-2, color)
    }

    /// Check if this is a foreground color (index -1).
    #[must_use]
    pub const fn is_foreground(&self) -> bool {
        self.index == -1
    }

    /// Check if this is a background color (index -2).
    #[must_use]
    pub const fn is_background(&self) -> bool {
        self.index == -2
    }

    /// Check if this is a standard palette color (index 0-255).
    #[must_use]
    pub const fn is_palette(&self) -> bool {
        self.index >= 0 && self.index <= 255
    }
}

/// Change or request color number `Pc` to the color specified by `Pt`.
///
/// *Sequence*: `OSC 4 ; Pc ; Pt ST`
///
/// Where `Pt` is either `?` to query or a color spec like `rgb:rr/gg/bb`.
#[derive(Debug, Clone, Copy, PartialEq, vtansi::derive::AnsiOutput)]
#[vtansi(osc, number = "4")]
pub struct RequestOrSetTerminalPaletteColor(
    /// The palette index.
    #[vtansi(flatten)]
    TerminalPaletteAction,
);

impl RequestOrSetTerminalPaletteColor {
    #[must_use]
    pub const fn query(index: i16) -> Self {
        Self(TerminalPaletteAction::query(index))
    }

    /// Create a query for the default foreground color (index -1).
    ///
    /// This is an iTerm2 extension.
    #[must_use]
    pub const fn query_foreground() -> Self {
        Self::query(-1)
    }

    /// Create a query for the default background color (index -2).
    ///
    /// This is an iTerm2 extension.
    #[must_use]
    pub const fn query_background() -> Self {
        Self::query(-2)
    }

    #[must_use]
    pub const fn set(index: i16, color: &XColor) -> Self {
        Self(TerminalPaletteAction::new(
            index,
            TerminalColorAction::Set(*color),
        ))
    }

    #[must_use]
    pub const fn set_foreground(color: &XColor) -> Self {
        Self::set(-1, color)
    }

    #[must_use]
    pub const fn set_background(color: &XColor) -> Self {
        Self::set(-2, color)
    }
}

impl Deref for RequestOrSetTerminalPaletteColor {
    type Target = TerminalPaletteAction;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Response to a query for the special text default foreground color.
///
/// *Sequence*: `OSC 10 ; Pt ST`
///
/// See <https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h3-Operating-System-Commands>
/// for reference and <https://terminalguide.namepad.de/seq/osc-10/> for
/// terminal support specifics.
#[derive(Debug, Clone, Copy, PartialEq, vtansi::derive::AnsiInput)]
#[vtansi(osc, number = "10")]
pub struct SpecialTextForegroundColorResponse(TerminalPaletteColor);

impl SpecialTextForegroundColorResponse {
    /// Create a new response with the given color.
    #[must_use]
    pub fn new(color: impl Into<TerminalPaletteColor>) -> Self {
        Self(color.into())
    }
}

impl Deref for SpecialTextForegroundColorResponse {
    type Target = TerminalPaletteColor;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Change/read special text default foreground color.
///
/// *Sequence*: `OSC 10 ; Pt ST`
///
/// This is a color in addition to the palette and direct colors which
/// applies to all text that has not otherwise been assigned a
/// foreground color.
///
/// See <https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h3-Operating-System-Commands>
/// for reference and <https://terminalguide.namepad.de/seq/osc-10/> for
/// terminal support specifics.
#[derive(Debug, Clone, Copy, PartialEq, vtansi::derive::AnsiOutput)]
#[vtansi(osc, number = "10")]
pub struct RequestOrSetSpecialTextForegroundColor(TerminalColorAction);

impl RequestOrSetSpecialTextForegroundColor {
    /// Create a query for the default foreground color.
    #[must_use]
    pub const fn query() -> Self {
        Self(TerminalColorAction::Query)
    }

    /// Create a command to set the default foreground color.
    #[must_use]
    pub const fn set(color: &XColor) -> Self {
        Self(TerminalColorAction::Set(*color))
    }
}

impl Deref for RequestOrSetSpecialTextForegroundColor {
    type Target = TerminalColorAction;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Response to a query for the special text default background color.
///
/// *Sequence*: `OSC 11 ; Pt ST`
///
/// See <https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h3-Operating-System-Commands>
/// for reference and <https://terminalguide.namepad.de/seq/osc-11/> for
/// terminal support specifics.
#[derive(Debug, Clone, Copy, PartialEq, vtansi::derive::AnsiInput)]
#[vtansi(osc, number = "11")]
pub struct SpecialTextBackgroundColorResponse(TerminalPaletteColor);

impl SpecialTextBackgroundColorResponse {
    /// Create a new response with the given color.
    #[must_use]
    pub fn new(color: impl Into<TerminalPaletteColor>) -> Self {
        Self(color.into())
    }
}

impl Deref for SpecialTextBackgroundColorResponse {
    type Target = TerminalPaletteColor;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Change/read special text default background color.
///
/// *Sequence*: `OSC 11 ; Pt ST`
///
/// This is a color in addition to the palette and direct colors which
/// applies to all text that has not otherwise been assigned a
/// background color.
///
/// See <https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h3-Operating-System-Commands>
/// for reference and <https://terminalguide.namepad.de/seq/osc-11/> for
/// terminal support specifics.
#[derive(Debug, Clone, Copy, PartialEq, vtansi::derive::AnsiOutput)]
#[vtansi(osc, number = "11")]
pub struct RequestOrSetSpecialTextBackgroundColor(TerminalColorAction);

impl RequestOrSetSpecialTextBackgroundColor {
    /// Create a query for the default background color.
    #[must_use]
    pub const fn query() -> Self {
        Self(TerminalColorAction::Query)
    }

    /// Create a command to set the default background color.
    #[must_use]
    pub const fn set(color: &XColor) -> Self {
        Self(TerminalColorAction::Set(*color))
    }
}

impl Deref for RequestOrSetSpecialTextBackgroundColor {
    type Target = TerminalColorAction;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Response to a query for the text cursor color.
///
/// *Sequence*: `OSC 12 ; Pt ST`
///
/// See <https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h3-Operating-System-Commands>
/// for reference and <https://terminalguide.namepad.de/seq/osc-12/> for
/// terminal support specifics.
#[derive(Debug, Clone, Copy, PartialEq, vtansi::derive::AnsiInput)]
#[vtansi(osc, number = "12")]
pub struct CursorColorResponse(TerminalPaletteColor);

impl Deref for CursorColorResponse {
    type Target = TerminalPaletteColor;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Change/read text cursor color.
///
/// *Sequence*: `OSC 12 ; Pt ST`
///
/// See <https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h3-Operating-System-Commands>
/// for reference and <https://terminalguide.namepad.de/seq/osc-12/> for
/// terminal support specifics.
#[derive(Debug, Clone, Copy, PartialEq, vtansi::derive::AnsiOutput)]
#[vtansi(osc, number = "12")]
pub struct RequestOrSetCursorColor(TerminalColorAction);

impl RequestOrSetCursorColor {
    /// Create a query for the cursor color.
    #[must_use]
    pub const fn query() -> Self {
        Self(TerminalColorAction::Query)
    }

    /// Create a command to set the cursor color.
    #[must_use]
    pub const fn set(color: &XColor) -> Self {
        Self(TerminalColorAction::Set(*color))
    }
}

/// Response to a query for the pointer (mouse cursor) foreground color.
///
/// *Sequence*: `OSC 13 ; Pt ST`
///
/// See <https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h3-Operating-System-Commands>
/// for reference and <https://terminalguide.namepad.de/seq/osc-13/> for
/// terminal support specifics.
#[derive(Debug, Clone, Copy, PartialEq, vtansi::derive::AnsiInput)]
#[vtansi(osc, number = "13")]
pub struct PointerForegroundColorResponse(TerminalPaletteColor);

impl Deref for PointerForegroundColorResponse {
    type Target = TerminalPaletteColor;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Change/read pointer (mouse cursor) foreground color.
///
/// *Sequence*: `OSC 13 ; Pt ST`
///
/// See <https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h3-Operating-System-Commands>
/// for reference and <https://terminalguide.namepad.de/seq/osc-13/> for
/// terminal support specifics.
#[derive(Debug, Clone, Copy, PartialEq, vtansi::derive::AnsiOutput)]
#[vtansi(osc, number = "13")]
pub struct RequestOrSetPointerForegroundColor(TerminalColorAction);

impl RequestOrSetPointerForegroundColor {
    /// Create a query for the pointer foreground color.
    #[must_use]
    pub const fn query() -> Self {
        Self(TerminalColorAction::Query)
    }

    /// Create a command to set the pointer foreground color.
    #[must_use]
    pub const fn set(color: &XColor) -> Self {
        Self(TerminalColorAction::Set(*color))
    }
}

/// Response to a query for the pointer (mouse cursor) background color.
///
/// *Sequence*: `OSC 14 ; Pt ST`
///
/// See <https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h3-Operating-System-Commands>
/// for reference and <https://terminalguide.namepad.de/seq/osc-14/> for
/// terminal support specifics.
#[derive(Debug, Clone, Copy, PartialEq, vtansi::derive::AnsiInput)]
#[vtansi(osc, number = "14")]
pub struct PointerBackgroundColorResponse(TerminalPaletteColor);

impl Deref for PointerBackgroundColorResponse {
    type Target = TerminalPaletteColor;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Change/read pointer (mouse cursor) background color.
///
/// *Sequence*: `OSC 14 ; Pt ST`
///
/// See <https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h3-Operating-System-Commands>
/// for reference and <https://terminalguide.namepad.de/seq/osc-14/> for
/// terminal support specifics.
#[derive(Debug, Clone, Copy, PartialEq, vtansi::derive::AnsiOutput)]
#[vtansi(osc, number = "14")]
pub struct RequestOrSetPointerBackgroundColor(TerminalColorAction);

impl RequestOrSetPointerBackgroundColor {
    /// Create a query for the pointer background color.
    #[must_use]
    pub const fn query() -> Self {
        Self(TerminalColorAction::Query)
    }

    /// Create a command to set the pointer background color.
    #[must_use]
    pub const fn set(color: &XColor) -> Self {
        Self(TerminalColorAction::Set(*color))
    }
}

/// Response to a query for the Tektronix foreground color.
///
/// *Sequence*: `OSC 15 ; Pt ST`
///
/// See <https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h3-Operating-System-Commands>
/// for reference and <https://terminalguide.namepad.de/seq/osc-15/> for
/// terminal support specifics.
#[derive(Debug, Clone, Copy, PartialEq, vtansi::derive::AnsiInput)]
#[vtansi(osc, number = "15")]
pub struct TektronixForegroundColorResponse(TerminalPaletteColor);

impl Deref for TektronixForegroundColorResponse {
    type Target = TerminalPaletteColor;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Change/read Tektronix foreground color.
///
/// *Sequence*: `OSC 15 ; Pt ST`
///
/// The Tektronix colors are initially set from the VT100 colors,
/// but after that can be set independently using these control sequences.
///
/// See <https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h3-Operating-System-Commands>
/// for reference and <https://terminalguide.namepad.de/seq/osc-15/> for
/// terminal support specifics.
#[derive(Debug, Clone, Copy, PartialEq, vtansi::derive::AnsiOutput)]
#[vtansi(osc, number = "15")]
pub struct RequestOrSetTektronixForegroundColor(TerminalColorAction);

impl RequestOrSetTektronixForegroundColor {
    /// Create a query for the Tektronix foreground color.
    #[must_use]
    pub const fn query() -> Self {
        Self(TerminalColorAction::Query)
    }

    /// Create a command to set the Tektronix foreground color.
    #[must_use]
    pub const fn set(color: &XColor) -> Self {
        Self(TerminalColorAction::Set(*color))
    }
}

/// Response to a query for the Tektronix background color.
///
/// *Sequence*: `OSC 16 ; Pt ST`
///
/// See <https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h3-Operating-System-Commands>
/// for reference and <https://terminalguide.namepad.de/seq/osc-16/> for
/// terminal support specifics.
#[derive(Debug, Clone, Copy, PartialEq, vtansi::derive::AnsiInput)]
#[vtansi(osc, number = "16")]
pub struct TektronixBackgroundColorResponse(TerminalPaletteColor);

impl Deref for TektronixBackgroundColorResponse {
    type Target = TerminalPaletteColor;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Change/read Tektronix background color.
///
/// *Sequence*: `OSC 16 ; Pt ST`
///
/// The Tektronix colors are initially set from the VT100 colors,
/// but after that can be set independently using these control sequences.
///
/// See <https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h3-Operating-System-Commands>
/// for reference and <https://terminalguide.namepad.de/seq/osc-16/> for
/// terminal support specifics.
#[derive(Debug, Clone, Copy, PartialEq, vtansi::derive::AnsiOutput)]
#[vtansi(osc, number = "16")]
pub struct RequestOrSetTektronixBackgroundColor(TerminalColorAction);

impl RequestOrSetTektronixBackgroundColor {
    /// Create a query for the Tektronix background color.
    #[must_use]
    pub const fn query() -> Self {
        Self(TerminalColorAction::Query)
    }

    /// Create a command to set the Tektronix background color.
    #[must_use]
    pub const fn set(color: &XColor) -> Self {
        Self(TerminalColorAction::Set(*color))
    }
}

/// Response to a query for the highlight (selection) background color.
///
/// *Sequence*: `OSC 17 ; Pt ST`
///
/// See <https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h3-Operating-System-Commands>
/// for reference and <https://terminalguide.namepad.de/seq/osc-17/> for
/// terminal support specifics.
#[derive(Debug, Clone, Copy, PartialEq, vtansi::derive::AnsiInput)]
#[vtansi(osc, number = "17")]
pub struct HighlightBackgroundColorResponse(TerminalPaletteColor);

impl Deref for HighlightBackgroundColorResponse {
    type Target = TerminalPaletteColor;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Change/read highlight (selection) background color.
///
/// *Sequence*: `OSC 17 ; Pt ST`
///
/// See <https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h3-Operating-System-Commands>
/// for reference and <https://terminalguide.namepad.de/seq/osc-17/> for
/// terminal support specifics.
#[derive(Debug, Clone, Copy, PartialEq, vtansi::derive::AnsiOutput)]
#[vtansi(osc, number = "17")]
pub struct RequestOrSetHighlightBackgroundColor(TerminalColorAction);

impl RequestOrSetHighlightBackgroundColor {
    /// Create a query for the highlight background color.
    #[must_use]
    pub const fn query() -> Self {
        Self(TerminalColorAction::Query)
    }

    /// Create a command to set the highlight background color.
    #[must_use]
    pub const fn set(color: &XColor) -> Self {
        Self(TerminalColorAction::Set(*color))
    }
}

/// Response to a query for the Tektronix cursor color.
///
/// *Sequence*: `OSC 18 ; Pt ST`
///
/// See <https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h3-Operating-System-Commands>
/// for reference and <https://terminalguide.namepad.de/seq/osc-18/> for
/// terminal support specifics.
#[derive(Debug, Clone, Copy, PartialEq, vtansi::derive::AnsiInput)]
#[vtansi(osc, number = "18")]
pub struct TektronixCursorColorResponse(TerminalPaletteColor);

impl Deref for TektronixCursorColorResponse {
    type Target = TerminalPaletteColor;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Change/read Tektronix cursor color.
///
/// *Sequence*: `OSC 18 ; Pt ST`
///
/// The Tektronix colors are initially set from the VT100 colors,
/// but after that can be set independently using these control sequences.
///
/// See <https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h3-Operating-System-Commands>
/// for reference and <https://terminalguide.namepad.de/seq/osc-18/> for
/// terminal support specifics.
#[derive(Debug, Clone, Copy, PartialEq, vtansi::derive::AnsiOutput)]
#[vtansi(osc, number = "18")]
pub struct RequestOrSetTektronixCursorColor(TerminalColorAction);

impl RequestOrSetTektronixCursorColor {
    /// Create a query for the Tektronix cursor color.
    #[must_use]
    pub const fn query() -> Self {
        Self(TerminalColorAction::Query)
    }

    /// Create a command to set the Tektronix cursor color.
    #[must_use]
    pub const fn set(color: &XColor) -> Self {
        Self(TerminalColorAction::Set(*color))
    }
}

/// Response to a query for the highlight (selection) foreground/text color.
///
/// *Sequence*: `OSC 19 ; Pt ST`
///
/// See <https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h3-Operating-System-Commands>
/// for reference and <https://terminalguide.namepad.de/seq/osc-19/> for
/// terminal support specifics.
#[derive(Debug, Clone, Copy, PartialEq, vtansi::derive::AnsiInput)]
#[vtansi(osc, number = "19")]
pub struct HighlightForegroundColorResponse(TerminalPaletteColor);

impl Deref for HighlightForegroundColorResponse {
    type Target = TerminalPaletteColor;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Change/read highlight (selection) foreground/text color.
///
/// *Sequence*: `OSC 19 ; Pt ST`
///
/// See <https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h3-Operating-System-Commands>
/// for reference and <https://terminalguide.namepad.de/seq/osc-19/> for
/// terminal support specifics.
#[derive(Debug, Clone, Copy, PartialEq, vtansi::derive::AnsiOutput)]
#[vtansi(osc, number = "19")]
pub struct RequestOrSetHighlightForegroundColor(TerminalColorAction);

impl RequestOrSetHighlightForegroundColor {
    /// Create a query for the highlight foreground color.
    #[must_use]
    pub const fn query() -> Self {
        Self(TerminalColorAction::Query)
    }

    /// Create a command to set the highlight foreground color.
    #[must_use]
    pub const fn set(color: &XColor) -> Self {
        Self(TerminalColorAction::Set(*color))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vtansi::encode::AnsiEncode;

    // TerminalPaletteColor tests

    #[test]
    fn test_terminal_palette_color_new() {
        let xcolor = XColor::from_rgb8(255, 128, 64);
        let color = TerminalPaletteColor::new(&xcolor);
        assert_eq!(*color, xcolor);
    }

    #[test]
    fn test_terminal_palette_color_from_xcolor() {
        let xcolor = XColor::rgb(0xffff, 0x8080, 0x0000);
        let color: TerminalPaletteColor = xcolor.into();
        assert_eq!(*color, xcolor);
    }

    #[test]
    fn test_terminal_palette_color_encode_rgb() {
        let color = TerminalPaletteColor(XColor::rgb(0xffff, 0x8080, 0x0000));
        let encoded = color.encode_ansi().unwrap();
        assert_eq!(encoded, b"rgb:ffff/8080/0000");
    }

    #[test]
    fn test_terminal_palette_color_encode_rgb_intensity() {
        let color = TerminalPaletteColor(XColor::rgb_intensity(1.0, 0.5, 0.0));
        let encoded = color.encode_ansi().unwrap();
        assert_eq!(encoded, b"rgbi:1.0/0.5/0.0");
    }

    #[test]
    fn test_terminal_palette_color_encode_cie_xyz() {
        let color = TerminalPaletteColor(XColor::cie_xyz(0.5, 0.3, 0.2));
        let encoded = color.encode_ansi().unwrap();
        assert_eq!(encoded, b"CIEXYZ:0.5/0.3/0.2");
    }

    #[test]
    fn test_terminal_palette_color_encode_cie_lab() {
        let color = TerminalPaletteColor(XColor::cie_lab(50.0, 25.0, -10.0));
        let encoded = color.encode_ansi().unwrap();
        assert_eq!(encoded, b"CIELab:50.0/25.0/-10.0");
    }

    #[test]
    fn test_terminal_palette_color_encode_cie_luv() {
        let color = TerminalPaletteColor(XColor::cie_luv(75.0, 30.0, -20.0));
        let encoded = color.encode_ansi().unwrap();
        assert_eq!(encoded, b"CIELuv:75.0/30.0/-20.0");
    }

    #[test]
    fn test_terminal_palette_color_encode_tek_hvc() {
        let color = TerminalPaletteColor(XColor::tek_hvc(180.0, 50.0, 25.0));
        let encoded = color.encode_ansi().unwrap();
        assert_eq!(encoded, b"TekHVC:180.0/50.0/25.0");
    }

    #[test]
    fn test_terminal_palette_color_encode_named_color() {
        // Named colors are parsed as RGB
        let xcolor: XColor = "red".parse().unwrap();
        let color = TerminalPaletteColor(xcolor);
        let encoded = color.encode_ansi().unwrap();
        assert_eq!(encoded, b"rgb:ffff/0000/0000");
    }

    #[test]
    fn test_terminal_palette_color_encode_named_color_dark_slate_gray() {
        let xcolor: XColor = "dark slate gray".parse().unwrap();
        let color = TerminalPaletteColor(xcolor);
        let encoded = color.encode_ansi().unwrap();
        // dark slate gray is RGB(47, 79, 79) -> expanded to 16-bit
        assert_eq!(encoded, b"rgb:2f2f/4f4f/4f4f");
    }

    // TerminalColorAction tests

    #[test]
    fn test_terminal_color_action_query() {
        let action = TerminalColorAction::query();
        assert_eq!(action, TerminalColorAction::Query);
    }

    #[test]
    fn test_terminal_color_action_set() {
        let xcolor = XColor::from_rgb8(255, 0, 0);
        let action = TerminalColorAction::set(xcolor);
        assert_eq!(action, TerminalColorAction::Set(xcolor));
    }

    #[test]
    fn test_terminal_color_action_from_xcolor() {
        let xcolor = XColor::from_rgb8(0, 255, 0);
        let action: TerminalColorAction = xcolor.into();
        assert_eq!(action, TerminalColorAction::Set(xcolor));
    }

    #[test]
    fn test_terminal_color_action_encode_query() {
        let action = TerminalColorAction::Query;
        let encoded = action.encode_ansi().unwrap();
        assert_eq!(encoded, b"?");
    }

    #[test]
    fn test_terminal_color_action_encode_set_rgb() {
        let action =
            TerminalColorAction::Set(XColor::rgb(0xffff, 0x0000, 0x8080));
        let encoded = action.encode_ansi().unwrap();
        assert_eq!(encoded, b"rgb:ffff/0000/8080");
    }

    #[test]
    fn test_terminal_color_action_encode_set_rgb_intensity() {
        let action =
            TerminalColorAction::Set(XColor::rgb_intensity(0.25, 0.5, 0.75));
        let encoded = action.encode_ansi().unwrap();
        assert_eq!(encoded, b"rgbi:0.25/0.5/0.75");
    }

    // TerminalPaletteAction tests

    #[test]
    fn test_terminal_palette_action_new() {
        let action = TerminalPaletteAction::new(5, TerminalColorAction::Query);
        assert_eq!(action.index, 5);
        assert!(action.is_query());
    }

    #[test]
    fn test_terminal_palette_action_query() {
        let action = TerminalPaletteAction::query(10);
        assert_eq!(action.index, 10);
        assert!(action.is_query());
        assert!(action.is_palette());
    }

    #[test]
    fn test_terminal_palette_action_query_foreground() {
        let action = TerminalPaletteAction::query_foreground();
        assert_eq!(action.index, -1);
        assert!(action.is_foreground());
        assert!(action.is_query());
    }

    #[test]
    fn test_terminal_palette_action_query_background() {
        let action = TerminalPaletteAction::query_background();
        assert_eq!(action.index, -2);
        assert!(action.is_background());
        assert!(action.is_query());
    }

    #[test]
    fn test_terminal_palette_action_set() {
        let color = XColor::from_rgb8(255, 128, 0);
        let action = TerminalPaletteAction::set(42, color);
        assert_eq!(action.index, 42);
        assert!(!action.is_query());
        assert_eq!(action.action, TerminalColorAction::Set(color));
    }

    #[test]
    fn test_terminal_palette_action_is_palette() {
        assert!(TerminalPaletteAction::query(0).is_palette());
        assert!(TerminalPaletteAction::query(255).is_palette());
        assert!(!TerminalPaletteAction::query(-1).is_palette());
        assert!(!TerminalPaletteAction::query(256).is_palette());
    }

    #[test]
    fn test_terminal_palette_action_encode_query() {
        let action = TerminalPaletteAction::query(5);
        let encoded = action.encode_ansi().unwrap();
        assert_eq!(encoded, b"5;?");
    }

    #[test]
    fn test_terminal_palette_action_encode_set() {
        let action =
            TerminalPaletteAction::set(10, XColor::rgb(0xffff, 0x0000, 0x0000));
        let encoded = action.encode_ansi().unwrap();
        assert_eq!(encoded, b"10;rgb:ffff/0000/0000");
    }

    #[test]
    fn test_terminal_palette_action_encode_set_rgb_intensity() {
        let action = TerminalPaletteAction::set(
            15,
            XColor::rgb_intensity(1.0, 0.0, 0.0),
        );
        let encoded = action.encode_ansi().unwrap();
        assert_eq!(encoded, b"15;rgbi:1.0/0.0/0.0");
    }

    // RequestOrSetTerminalPaletteColor tests (AnsiOutput)

    #[test]
    fn test_request_palette_color_query() {
        let request = RequestOrSetTerminalPaletteColor::query(5);
        let encoded = request.encode_ansi().unwrap();
        assert_eq!(encoded, b"\x1b]4;5;?\x1b\\");
    }

    #[test]
    fn test_request_palette_color_query_foreground() {
        let request = RequestOrSetTerminalPaletteColor::query_foreground();
        let encoded = request.encode_ansi().unwrap();
        assert_eq!(encoded, b"\x1b]4;-1;?\x1b\\");
    }

    #[test]
    fn test_request_palette_color_query_background() {
        let request = RequestOrSetTerminalPaletteColor::query_background();
        let encoded = request.encode_ansi().unwrap();
        assert_eq!(encoded, b"\x1b]4;-2;?\x1b\\");
    }

    #[test]
    fn test_request_palette_color_set_rgb() {
        let color = XColor::rgb(0xffff, 0x8080, 0x0000);
        let request = RequestOrSetTerminalPaletteColor::set(10, &color);
        let encoded = request.encode_ansi().unwrap();
        assert_eq!(encoded, b"\x1b]4;10;rgb:ffff/8080/0000\x1b\\");
    }

    #[test]
    fn test_request_palette_color_set_rgb_intensity() {
        let color = XColor::rgb_intensity(1.0, 0.5, 0.25);
        let request = RequestOrSetTerminalPaletteColor::set(20, &color);
        let encoded = request.encode_ansi().unwrap();
        assert_eq!(encoded, b"\x1b]4;20;rgbi:1.0/0.5/0.25\x1b\\");
    }

    #[test]
    fn test_request_palette_color_set_foreground() {
        let color = XColor::from_rgb8(255, 255, 255);
        let request = RequestOrSetTerminalPaletteColor::set_foreground(&color);
        let encoded = request.encode_ansi().unwrap();
        assert_eq!(encoded, b"\x1b]4;-1;rgb:ffff/ffff/ffff\x1b\\");
    }

    #[test]
    fn test_request_palette_color_set_background() {
        let color = XColor::from_rgb8(0, 0, 0);
        let request = RequestOrSetTerminalPaletteColor::set_background(&color);
        let encoded = request.encode_ansi().unwrap();
        assert_eq!(encoded, b"\x1b]4;-2;rgb:0000/0000/0000\x1b\\");
    }

    #[test]
    fn test_request_palette_color_set_named_color() {
        let color: XColor = "cornflower blue".parse().unwrap();
        let request = RequestOrSetTerminalPaletteColor::set(100, &color);
        let encoded = request.encode_ansi().unwrap();
        // cornflower blue is RGB(100, 149, 237) -> expanded to 16-bit
        assert_eq!(encoded, b"\x1b]4;100;rgb:6464/9595/eded\x1b\\");
    }

    // RequestOrSetDefaultForegroundColor tests (AnsiOutput)

    #[test]
    fn test_request_default_foreground_color_query() {
        let request = RequestOrSetSpecialTextForegroundColor::query();
        let encoded = request.encode_ansi().unwrap();
        assert_eq!(encoded, b"\x1b]10;?\x1b\\");
    }

    #[test]
    fn test_request_default_foreground_color_set_rgb() {
        let color = XColor::rgb(0xffff, 0x8080, 0x0000);
        let request = RequestOrSetSpecialTextForegroundColor::set(&color);
        let encoded = request.encode_ansi().unwrap();
        assert_eq!(encoded, b"\x1b]10;rgb:ffff/8080/0000\x1b\\");
    }

    #[test]
    fn test_request_default_foreground_color_set_named() {
        let color: XColor = "white".parse().unwrap();
        let request = RequestOrSetSpecialTextForegroundColor::set(&color);
        let encoded = request.encode_ansi().unwrap();
        assert_eq!(encoded, b"\x1b]10;rgb:ffff/ffff/ffff\x1b\\");
    }

    // RequestOrSetDefaultBackgroundColor tests (AnsiOutput)

    #[test]
    fn test_request_default_background_color_query() {
        let request = RequestOrSetSpecialTextBackgroundColor::query();
        let encoded = request.encode_ansi().unwrap();
        assert_eq!(encoded, b"\x1b]11;?\x1b\\");
    }

    #[test]
    fn test_request_default_background_color_set_rgb() {
        let color = XColor::rgb(0x0000, 0x0000, 0x0000);
        let request = RequestOrSetSpecialTextBackgroundColor::set(&color);
        let encoded = request.encode_ansi().unwrap();
        assert_eq!(encoded, b"\x1b]11;rgb:0000/0000/0000\x1b\\");
    }

    #[test]
    fn test_request_default_background_color_set_rgb_intensity() {
        let color = XColor::rgb_intensity(0.1, 0.2, 0.3);
        let request = RequestOrSetSpecialTextBackgroundColor::set(&color);
        let encoded = request.encode_ansi().unwrap();
        assert_eq!(encoded, b"\x1b]11;rgbi:0.1/0.2/0.3\x1b\\");
    }

    // RequestOrSetCursorColor tests (AnsiOutput)

    #[test]
    fn test_request_cursor_color_query() {
        let request = RequestOrSetCursorColor::query();
        let encoded = request.encode_ansi().unwrap();
        assert_eq!(encoded, b"\x1b]12;?\x1b\\");
    }

    #[test]
    fn test_request_cursor_color_set_rgb() {
        let color = XColor::rgb(0xffff, 0x0000, 0x0000);
        let request = RequestOrSetCursorColor::set(&color);
        let encoded = request.encode_ansi().unwrap();
        assert_eq!(encoded, b"\x1b]12;rgb:ffff/0000/0000\x1b\\");
    }

    // RequestOrSetPointerForegroundColor tests (AnsiOutput)

    #[test]
    fn test_request_pointer_foreground_color_query() {
        let request = RequestOrSetPointerForegroundColor::query();
        let encoded = request.encode_ansi().unwrap();
        assert_eq!(encoded, b"\x1b]13;?\x1b\\");
    }

    #[test]
    fn test_request_pointer_foreground_color_set_rgb() {
        let color = XColor::rgb(0x0000, 0xffff, 0x0000);
        let request = RequestOrSetPointerForegroundColor::set(&color);
        let encoded = request.encode_ansi().unwrap();
        assert_eq!(encoded, b"\x1b]13;rgb:0000/ffff/0000\x1b\\");
    }

    // RequestOrSetPointerBackgroundColor tests (AnsiOutput)

    #[test]
    fn test_request_pointer_background_color_query() {
        let request = RequestOrSetPointerBackgroundColor::query();
        let encoded = request.encode_ansi().unwrap();
        assert_eq!(encoded, b"\x1b]14;?\x1b\\");
    }

    #[test]
    fn test_request_pointer_background_color_set_rgb() {
        let color = XColor::rgb(0x0000, 0x0000, 0xffff);
        let request = RequestOrSetPointerBackgroundColor::set(&color);
        let encoded = request.encode_ansi().unwrap();
        assert_eq!(encoded, b"\x1b]14;rgb:0000/0000/ffff\x1b\\");
    }

    // RequestOrSetTektronixForegroundColor tests (AnsiOutput)

    #[test]
    fn test_request_tektronix_foreground_color_query() {
        let request = RequestOrSetTektronixForegroundColor::query();
        let encoded = request.encode_ansi().unwrap();
        assert_eq!(encoded, b"\x1b]15;?\x1b\\");
    }

    #[test]
    fn test_request_tektronix_foreground_color_set_rgb() {
        let color = XColor::rgb(0x1111, 0x2222, 0x3333);
        let request = RequestOrSetTektronixForegroundColor::set(&color);
        let encoded = request.encode_ansi().unwrap();
        assert_eq!(encoded, b"\x1b]15;rgb:1111/2222/3333\x1b\\");
    }

    // RequestOrSetTektronixBackgroundColor tests (AnsiOutput)

    #[test]
    fn test_request_tektronix_background_color_query() {
        let request = RequestOrSetTektronixBackgroundColor::query();
        let encoded = request.encode_ansi().unwrap();
        assert_eq!(encoded, b"\x1b]16;?\x1b\\");
    }

    #[test]
    fn test_request_tektronix_background_color_set_rgb() {
        let color = XColor::rgb(0x4444, 0x5555, 0x6666);
        let request = RequestOrSetTektronixBackgroundColor::set(&color);
        let encoded = request.encode_ansi().unwrap();
        assert_eq!(encoded, b"\x1b]16;rgb:4444/5555/6666\x1b\\");
    }

    // RequestOrSetHighlightBackgroundColor tests (AnsiOutput)

    #[test]
    fn test_request_highlight_background_color_query() {
        let request = RequestOrSetHighlightBackgroundColor::query();
        let encoded = request.encode_ansi().unwrap();
        assert_eq!(encoded, b"\x1b]17;?\x1b\\");
    }

    #[test]
    fn test_request_highlight_background_color_set_rgb() {
        let color = XColor::rgb(0x7777, 0x8888, 0x9999);
        let request = RequestOrSetHighlightBackgroundColor::set(&color);
        let encoded = request.encode_ansi().unwrap();
        assert_eq!(encoded, b"\x1b]17;rgb:7777/8888/9999\x1b\\");
    }

    // RequestOrSetTektronixCursorColor tests (AnsiOutput)

    #[test]
    fn test_request_tektronix_cursor_color_query() {
        let request = RequestOrSetTektronixCursorColor::query();
        let encoded = request.encode_ansi().unwrap();
        assert_eq!(encoded, b"\x1b]18;?\x1b\\");
    }

    #[test]
    fn test_request_tektronix_cursor_color_set_rgb() {
        let color = XColor::rgb(0xaaaa, 0xbbbb, 0xcccc);
        let request = RequestOrSetTektronixCursorColor::set(&color);
        let encoded = request.encode_ansi().unwrap();
        assert_eq!(encoded, b"\x1b]18;rgb:aaaa/bbbb/cccc\x1b\\");
    }

    // RequestOrSetHighlightForegroundColor tests (AnsiOutput)

    #[test]
    fn test_request_highlight_foreground_color_query() {
        let request = RequestOrSetHighlightForegroundColor::query();
        let encoded = request.encode_ansi().unwrap();
        assert_eq!(encoded, b"\x1b]19;?\x1b\\");
    }

    #[test]
    fn test_request_highlight_foreground_color_set_rgb() {
        let color = XColor::rgb(0xdddd, 0xeeee, 0xffff);
        let request = RequestOrSetHighlightForegroundColor::set(&color);
        let encoded = request.encode_ansi().unwrap();
        assert_eq!(encoded, b"\x1b]19;rgb:dddd/eeee/ffff\x1b\\");
    }
}
