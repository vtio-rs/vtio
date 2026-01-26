//! Buffer control/information messages.

use crate::terminal_mode;

terminal_mode!(
    /// Insert mode (`IRM`).
    ///
    /// # Sequence
    ///
    /// `CSI 4 h` (set) / `CSI 4 l` (reset)
    ///
    /// When enabled, newly printed characters are inserted at the cursor
    /// position, shifting existing characters to the right.
    ///
    /// See <https://terminalguide.namepad.de/mode/p4/> for terminal
    /// support specifics.
    InsertMode, params = ["4"]
);

terminal_mode!(
    /// Cursor blinking mode (`ATT610_BLINK`).
    ///
    /// # Sequence
    ///
    /// `CSI 12 h` (set) / `CSI 12 l` (reset)
    ///
    /// If set, the cursor is blinking.
    ///
    /// See also select cursor style for a more widely supported
    /// alternative.
    ///
    /// See <https://terminalguide.namepad.de/mode/p12/> for terminal
    /// support specifics.
    EchoMode, params = ["12"]
);

terminal_mode!(
    /// Linefeed/Newline mode (`LNM`).
    ///
    /// # Sequence
    ///
    /// `CSI 20 h` (set) / `CSI 20 l` (reset)
    ///
    /// Controls whether line feed characters also perform a carriage
    /// return.
    ///
    /// See <https://terminalguide.namepad.de/mode/p20/> for terminal
    /// support specifics.
    LinefeedMode, params = ["20"]
);

terminal_mode!(
    /// Reserved for VT52 emulators (`DECANM`).
    ///
    /// # Sequence
    ///
    /// `CSI ? 2 h` (set) / `CSI ? 2 l` (reset)
    ///
    /// Reserved for VT52 emulation.
    ///
    /// See <https://terminalguide.namepad.de/mode/p2/> for terminal
    /// support specifics.
    VT52Mode, private = '?', params = ["2"]
);

terminal_mode!(
    /// 132 column mode (`DECCOLM`).
    ///
    /// # Sequence
    ///
    /// `CSI ? 3 h` (set) / `CSI ? 3 l` (reset)
    ///
    /// Change terminal width between 80 and 132 column mode.
    ///
    /// This mode only is supported when enable support for 132 column
    /// mode is set.
    ///
    /// Modern terminals don't have a fixed width and users generally
    /// expect the terminal to keep the size they assigned to the
    /// terminal. This control violates that expectation.
    ///
    /// If set the terminal is resized to 132 columns wide. If unset
    /// the terminal is resized to 80 columns wide.
    ///
    /// If do not clear screen on 132 column mode change is not set,
    /// the screen is cleared.
    ///
    /// The cursor is moved as invoking set cursor position with
    /// `column` and `row` set to 1.
    ///
    /// If the mode is set, left and right margin is reset.
    ///
    /// See <https://terminalguide.namepad.de/mode/p3/> for terminal
    /// support specifics.
    HundredThirtyTwoColumnMode, private = '?', params = ["3"]
);

terminal_mode!(
    /// Enable support for 132 column mode (`132COLS`).
    ///
    /// # Sequence
    ///
    /// `CSI ? 40 h` (set) / `CSI ? 40 l` (reset)
    ///
    /// Enables support for 132 column mode.
    ///
    /// See <https://terminalguide.namepad.de/mode/p40/> for terminal
    /// support specifics.
    EnableSupportForHundredThirtyTwoColumnMode, private = '?', params = ["40"]
);

terminal_mode!(
    /// Do not clear screen on 132 column mode change (`DECNCSM`).
    ///
    /// # Sequence
    ///
    /// `CSI ? 95 h` (set) / `CSI ? 95 l` (reset)
    ///
    /// Do not clear screen on change of 132 column mode.
    ///
    /// Only available in xterm VT level 5 or above (non-default level).
    ///
    /// See <https://terminalguide.namepad.de/mode/p95/> for terminal
    /// support specifics.
    KeepScreenOnHundredThirtyTwoColumnChangeMode, private = '?', params = ["95"]
);

terminal_mode!(
    /// Reverse display colors (`DECSCNM`).
    ///
    /// # Sequence
    ///
    /// `CSI ? 5 h` (set) / `CSI ? 5 l` (reset)
    ///
    /// Reverses the foreground and background colors of some cells.
    ///
    /// Exact behavior is implementation specific. Most terminals swap
    /// default (unnamed) background and foreground colors when
    /// rendering.
    ///
    /// See <https://terminalguide.namepad.de/mode/p5/> for terminal
    /// support specifics.
    ReverseDisplayColorsMode, private = '?', params = ["5"]
);

terminal_mode!(
    /// Wraparound mode (`DECAWM`).
    ///
    /// # Sequence
    ///
    /// `CSI ? 7 h` (set) / `CSI ? 7 l` (reset)
    ///
    /// Enable or disable automatic line wrapping.
    ///
    /// If disabled, cursor will stop advancing on right-most column of
    /// the scroll region or screen. Printing additional characters will
    /// (repeatedly) overwrite the cell at the cursor position.
    ///
    /// If enabled, printing to the last cell in the scroll region or
    /// screen will leave the cursor at that cell and set the pending
    /// wrap state of the cursor. Printing while the pending wrap state
    /// of the cursor is set will wrap back to the left-most column in
    /// the scroll region, unset the pending wrap state and invoke
    /// index. In some terminals it also saves the information that the
    /// line was wrapped for resize and clipboard heuristics.
    ///
    /// See <https://terminalguide.namepad.de/mode/p7/> for terminal
    /// support specifics.
    LineWraparoundMode, private = '?', params = ["7"]
);

terminal_mode!(
    /// Scrollbar visibility (`RXVT_SCROLLBAR`).
    ///
    /// # Sequence
    ///
    /// `CSI ? 30 h` (set) / `CSI ? 30 l` (reset)
    ///
    /// Show scrollbar.
    ///
    /// See <https://terminalguide.namepad.de/mode/p30/> for terminal
    /// support specifics.
    ScrollbarVisibilityMode, private = '?', params = ["30"]
);

terminal_mode!(
    /// Alternate screen buffer (`ALTBUF`).
    ///
    /// # Sequence
    ///
    /// `CSI ? 47 h` (set) / `CSI ? 47 l` (reset)
    ///
    /// Switch to alternate screen buffer.
    ///
    /// Terminals supporting this mode offer an alternate screen buffer
    /// in addition to the primary buffer. The primary buffer usually
    /// supports scroll-back. The alternate buffer is for full screen
    /// applications. It does not support scroll-back (or displays
    /// scroll-back from the primary screen). Switching to the alternate
    /// screen buffer for fullscreen applications allows visually
    /// switching back to the contents of the primary buffer after the
    /// application terminates.
    ///
    /// Both buffers are partially independent. They have a separate
    /// cell matrix and cursor save state.
    ///
    /// See <https://terminalguide.namepad.de/mode/p47/> for terminal
    /// support specifics.
    AlternateScreenBasicMode, private = '?', params = ["47"]
);

terminal_mode!(
    /// Alternate screen buffer with clear on exit.
    ///
    /// # Sequence
    ///
    /// `CSI ? 1047 h` (set) / `CSI ? 1047 l` (reset)
    ///
    /// Like alternate screen buffer but clears the alternate buffer on
    /// reset.
    ///
    /// The clear of the alternate buffer fills all cells in the
    /// alternate buffer with space and the current SGR state.
    ///
    /// Leaving this mode might clear the text selection in terminals
    /// that support copy and paste.
    ///
    /// See <https://terminalguide.namepad.de/mode/p1047/> for terminal
    /// support specifics.
    AlternateScreenClearOnExitMode, private = '?', params = ["1047"]
);

terminal_mode!(
    /// Alternate screen buffer with cursor save and clear on enter.
    ///
    /// # Sequence
    ///
    /// `CSI ? 1049 h` (set) / `CSI ? 1049 l` (reset)
    ///
    /// Like alternate screen buffer but saves the cursor and clears the
    /// alternate buffer on activation.
    ///
    /// The clear of the alternate buffer fills all cells in the
    /// alternate buffer with space and the current SGR state.
    ///
    /// The cursor is saved before switching to alternate mode as if
    /// save cursor was invoked. On reset the cursor is restored after
    /// switching to the primary screen buffer as if restore cursor was
    /// invoked.
    ///
    /// See <https://terminalguide.namepad.de/mode/p1049/> for terminal
    /// support specifics.
    AlternateScreenMode, private = '?', params = ["1049"]
);

terminal_mode!(
    /// Report focus change.
    ///
    /// # Sequence
    ///
    /// `CSI ? 1004 h` (set) / `CSI ? 1004 l` (reset)
    ///
    /// When the terminal gains focus emit `ESC [ I`.
    ///
    /// When the terminal loses focus emit `ESC [ O`.
    ///
    /// See <https://terminalguide.namepad.de/mode/p1004/> for terminal
    /// support specifics.
    ReportFocusChangeMode, private = '?', params = ["1004"]
);

terminal_mode!(
    /// Inhibit scroll on application output.
    ///
    /// # Sequence
    ///
    /// `CSI ? 1010 h` (set) / `CSI ? 1010 l` (reset)
    ///
    /// Disable automatic scroll to bottom when the application outputs
    /// a printable character.
    ///
    /// Note: xterm implements inverted behavior.
    ///
    /// See <https://terminalguide.namepad.de/mode/p1010/> for terminal
    /// support specifics.
    InhibitScrollOnApplicationOutputMode, private = '?', params = ["1010"]
);

terminal_mode!(
    /// Scroll on keyboard input.
    ///
    /// # Sequence
    ///
    /// `CSI ? 1011 h` (set) / `CSI ? 1011 l` (reset)
    ///
    /// If set, scrolls to the bottom on every keypress.
    ///
    /// See <https://terminalguide.namepad.de/mode/p1011/> for terminal
    /// support specifics.
    ScrollOnKeyboardInputMode, private = '?', params = ["1011"]
);

terminal_mode!(
    /// Bold/blinking cells are also bright.
    ///
    /// # Sequence
    ///
    /// `CSI ? 1021 h` (set) / `CSI ? 1021 l` (reset)
    ///
    /// If a cell is rendered in bold, and its foreground color is one
    /// of the 8 'named' dark colors, force that cell's foreground to be
    /// its corresponding bright named color.
    ///
    /// If a cell is rendered as blinking, and its background color is
    /// one of the 8 'named' dark colors, force that cell's background
    /// to be its corresponding bright named color.
    ///
    /// See <https://terminalguide.namepad.de/mode/p1021/> for terminal
    /// support specifics.
    BoldBlinkingBrightMode, private = '?', params = ["1021"]
);

terminal_mode!(
    /// Bracketed paste mode.
    ///
    /// # Sequence
    ///
    /// `CSI ? 2004 h` (set) / `CSI ? 2004 l` (reset)
    ///
    /// Bracket clipboard paste contents in delimiter sequences.
    ///
    /// When pasting from the (e.g. system) clipboard add `ESC [ 200 ~`
    /// before the clipboard contents and `ESC [ 201 ~` after the
    /// clipboard contents. This allows applications to distinguish
    /// clipboard contents from manually typed text.
    ///
    /// See <https://terminalguide.namepad.de/mode/p2004/> for terminal
    /// support specifics.
    BracketedPasteMode, private = '?', params = ["2004"]
);

terminal_mode!(
    /// Synchronized update mode.
    ///
    /// # Sequence
    ///
    /// `CSI ? 2026 h` (set) / `CSI ? 2026 l` (reset)
    ///
    /// When the synchronization mode is enabled following render calls
    /// will keep rendering the last rendered state. The terminal
    /// keeps processing incoming text and sequences. When the
    /// synchronized update mode is disabled again the renderer may fetch
    /// the latest screen buffer state again, effectively avoiding the
    /// tearing effect by unintentionally rendering in the middle a of
    /// an application screen update.
    ///
    /// See <https://gitlab.com/gnachman/iterm2/-/wikis/synchronized-updates-spec>
    /// for more details and <https://terminalguide.namepad.de/mode/p2026/>
    /// for terminal support specifics.
    SynchronizedUpdateMode, private = '?', params = ["2006"]
);

terminal_mode!(
    /// Request unsolicited DSR on color palette updates.
    ///
    /// # Sequence
    ///
    /// `CSI ? 2031 h` (set) / `CSI ? 2031 l` (reset)
    ///
    /// See <https://contour-terminal.org/vt-extensions/color-palette-update-notifications/>
    /// for more details.
    UnsolicitedColorPaletteReportMode, private = '?', params = ["2031"]
);

/// Bracketed paste start.
///
/// *Sequence*: `CSI 200 ~`
#[derive(
    Debug,
    PartialOrd,
    PartialEq,
    Eq,
    Clone,
    Copy,
    Hash,
    vtansi::derive::AnsiInput,
)]
#[vtansi(csi, params = ["200"], finalbyte = '~')]
pub struct BracketedPasteStart;

/// Bracketed paste end.
///
/// *Sequence*: `CSI 201 ~`
#[derive(
    Debug,
    PartialOrd,
    PartialEq,
    Eq,
    Clone,
    Copy,
    Hash,
    vtansi::derive::AnsiInput,
)]
#[vtansi(csi, params = ["201"], finalbyte = '~')]
pub struct BracketedPasteEnd;

/// Bracketed paste.
#[derive(Debug, PartialOrd, PartialEq, Eq, Clone, Copy, Hash)]
pub struct BracketedPaste<'a>(pub &'a [u8]);

better_any::tid! {BracketedPaste<'a>}

impl vtansi::TerseDisplay for BracketedPaste<'_> {
    fn terse_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

impl vtansi::AnsiEncode for BracketedPaste<'_> {
    #[inline]
    fn encode_ansi_into<W: std::io::Write + ?Sized>(
        &self,
        sink: &mut W,
    ) -> Result<usize, vtansi::EncodeError> {
        let mut count = BracketedPasteStart.encode_ansi_into(sink)?;
        count += vtansi::write_bytes_into(sink, self.0)?;
        count += BracketedPasteEnd.encode_ansi_into(sink)?;
        Ok(count)
    }
}

impl<'a> vtansi::AnsiEvent<'a> for BracketedPaste<'a> {
    fn ansi_control_kind(&self) -> Option<vtansi::AnsiControlFunctionKind> {
        None
    }

    fn ansi_direction(&self) -> vtansi::AnsiControlDirection {
        vtansi::AnsiControlDirection::Input
    }

    vtansi::impl_ansi_event_encode!();
    vtansi::impl_ansi_event_terse_fmt!();
}

/// Bell (BEL).
///
/// *Sequence*: `0x07` (C0 control code)
///
/// Traditionally rings a bell.
///
/// Current implementations vary in how this is interpreted. Most
/// implementations still support an audible signal but often also offer
/// setting window manager urgency hints or other advanced reactions.
///
/// See <https://terminalguide.namepad.de/seq/a_c0-g/> for terminal
/// support specifics.
#[derive(
    Debug,
    PartialOrd,
    PartialEq,
    Eq,
    Clone,
    Copy,
    Hash,
    vtansi::derive::AnsiOutput,
)]
#[vtansi(c0, code = 0x07)]
pub struct Bell;

/// Request text attributes (SGR) using `DECRQSS`.
///
/// *Sequence*: `DCS $ q m ST`
///
/// Query SGR state using DEC Request Status String.
///
/// The terminal replies with `DCS 1 $ r Ps ; Ps ; ... m ST` containing
/// the current SGR attributes.
#[derive(
    Debug,
    PartialOrd,
    PartialEq,
    Eq,
    Clone,
    Copy,
    Hash,
    vtansi::derive::AnsiOutput,
)]
#[vtansi(dcs, intermediate = "$", finalbyte = 'q', data = "m")]
pub struct RequestTextAttributes;

/// Full Reset (`RIS`).
///
/// *Sequence*: `ESC c`
///
/// Full reset of the terminal state.
///
/// This resets palette colors, switches to primary screen, clears the
/// screen and scrollback buffer, moves cursor to (1, 1), resets SGR
/// attributes, makes cursor visible, resets cursor shape and
/// blinking, resets cursor origin mode, resets scrolling region,
/// resets character sets, disables all mouse tracking modes, resets
/// tab stops, and reverts many other terminal settings to their
/// initial state.
///
/// See <https://terminalguide.namepad.de/seq/a_esc_sc/> for terminal
/// support specifics.
#[derive(
    Debug,
    PartialOrd,
    PartialEq,
    Eq,
    Clone,
    Copy,
    Hash,
    vtansi::derive::AnsiOutput,
)]
#[vtansi(esc, finalbyte = 'c')]
pub struct FullReset;

/// Request Terminal ID (`DECID`).
///
/// *Sequence*: `ESC Z`
///
/// Same as primary device attributes without parameters.
///
/// See <https://terminalguide.namepad.de/seq/a_esc_cz/> for terminal
/// support specifics.
#[derive(
    Debug,
    PartialOrd,
    PartialEq,
    Eq,
    Clone,
    Copy,
    Hash,
    vtansi::derive::AnsiOutput,
)]
#[vtansi(esc, finalbyte = 'Z')]
pub struct RequestTerminalID;

/// Request primary device attributes (`DA1`).
///
/// *Sequence*: `CSI c` or `CSI 0 c`
///
/// Query the terminal's primary device attributes.
///
/// The terminal responds with `CSI ? Ps ; Ps ; ... c` where the first
/// `Ps` is the conformance level and subsequent parameters indicate
/// supported capabilities.
///
/// See <https://terminalguide.namepad.de/seq/csi_sc/> for terminal
/// support specifics.
#[derive(
    Debug,
    Default,
    PartialOrd,
    PartialEq,
    Eq,
    Clone,
    Copy,
    Hash,
    vtansi::derive::AnsiOutput,
)]
#[vtansi(csi, finalbyte = 'c')]
pub struct RequestPrimaryDeviceAttributes(Option<u8>);

impl RequestPrimaryDeviceAttributes {
    #[must_use]
    pub fn new() -> Self {
        Self(Some(0))
    }
}

/// Request secondary device attributes (`DA2`).
///
/// *Sequence*: `CSI > c` or `CSI > 0 c`
///
/// Query the terminal's secondary device attributes.
///
/// The terminal responds with `CSI > Ps ; Ps ; Ps c` containing
/// terminal type, firmware version, and ROM cartridge registration number.
///
/// See <https://terminalguide.namepad.de/seq/> for terminal support
/// specifics.
#[derive(
    Debug,
    PartialOrd,
    PartialEq,
    Eq,
    Clone,
    Copy,
    Hash,
    vtansi::derive::AnsiOutput,
)]
#[vtansi(csi, intermediate = ">", finalbyte = 'c')]
pub struct RequestSecondaryDeviceAttributes;

/// Request tertiary device attributes (`DA3`).
///
/// *Sequence*: `CSI = 0 c`
///
/// Query the terminal's tertiary device attributes.
///
/// The terminal responds with `DCS ! | D...D ST` where `D...D` is the
/// unit ID as a hex string.
///
/// See <https://terminalguide.namepad.de/seq/csi_sc__r/> for terminal support
/// specifics.
#[derive(
    Debug,
    PartialOrd,
    PartialEq,
    Eq,
    Clone,
    Copy,
    Hash,
    vtansi::derive::AnsiOutput,
)]
#[vtansi(csi, params = ["=0"], finalbyte = 'c')]
pub struct RequestTerminalUnitId;

/// Terminal conformance level for DA1 response.
///
/// The first parameter in a DA1 response indicates the terminal's
/// conformance level.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    vtansi::derive::ToAnsi,
    vtansi::derive::FromAnsi,
    num_enum::TryFromPrimitive,
    num_enum::IntoPrimitive,
)]
#[repr(u8)]
pub enum ConformanceLevel {
    /// VT100 compatibility (Level 1).
    VT100 = 1,
    /// VT102 compatibility (Level 1).
    VT102 = 6,
    /// VT220 compatibility (Level 2).
    VT220 = 62,
    /// VT320 compatibility (Level 3).
    VT320 = 63,
    /// VT420/VT510/VT525 compatibility (Level 4).
    VT420 = 64,
    /// Other unrecognized value.
    #[num_enum(catch_all)]
    Other(u8),
}

/// Terminal capability flags for DA1 response.
///
/// These flags indicate which features the terminal supports.
/// Multiple capabilities can be combined in a single response.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    vtansi::derive::ToAnsi,
    vtansi::derive::FromAnsi,
    num_enum::TryFromPrimitive,
    num_enum::IntoPrimitive,
)]
#[repr(u8)]
pub enum TerminalCapability {
    /// 132 columns mode (`DECCOLM`).
    Columns132 = 1,
    /// Printer port.
    Printer = 2,
    /// `ReGIS` graphics.
    ReGISGraphics = 3,
    /// `SIXEL` graphics.
    SixelGraphics = 4,
    /// Selective erase (`DECSED`, `DECSEL`).
    SelectiveErase = 6,
    /// Soft character sets (`DRCS` - Dynamic Redefinable Character Sets).
    SoftCharacterSets = 7,
    /// User-defined keys (`DECUDK`).
    UserDefinedKeys = 8,
    /// National Replacement Character sets (`NRC`).
    NationalReplacementCharsets = 9,
    /// Blink attribute (`SGR 5`).
    Blink = 12,
    /// Technical character set.
    TechnicalCharset = 15,
    /// Locator (Mouse) device.
    LocatorDevice = 16,
    /// User-defined keys (extended).
    UserDefinedKeysExtended = 17,
    /// National Replacement Character sets (extended).
    NationalReplacementCharsetsExtended = 18,
    /// 24 or more lines.
    MoreThan24Lines = 19,
    /// Multiple pages / horizontal scrolling.
    HorizontalScrolling = 21,
    /// ANSI color support.
    Color = 22,
    /// Soft key labels.
    SoftKeyLabels = 23,
    /// Rectangular area operations (`DECCRA`, `DECFRA`).
    RectangularAreaOps = 24,
    /// Locator events (motion/button).
    LocatorEvents = 29,
    /// Windowing extensions (`DECRQCRA`).
    WindowingExtensions = 42,
    /// Cursor position report format.
    CursorPositionReportFormat = 44,
    /// RGB color / extended color.
    ExtendedColor = 46,
    /// xterm/VT525-like (older xterm-style)
    VT525Xterm = 52,
    /// Modern xterm/VT525-like
    VT525ModernXterm = 67,
    /// Other unrecognized value.
    #[num_enum(catch_all)]
    Other(u8),
}

/// Terminal capabilities wrapper for encoding.
///
/// Encodes a vector of terminal capabilities as a semicolon-separated list.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Capabilities(pub Vec<TerminalCapability>);

impl Capabilities {
    /// Create from a vector of terminal capabilities.
    #[must_use]
    pub fn new(capabilities: Vec<TerminalCapability>) -> Self {
        Self(capabilities)
    }

    /// Create from a slice of terminal capabilities.
    #[must_use]
    pub fn from_slice(capabilities: &[TerminalCapability]) -> Self {
        Self(capabilities.to_vec())
    }
}

impl vtansi::AnsiEncode for Capabilities {
    fn encode_ansi_into<W: std::io::Write + ?Sized>(
        &self,
        sink: &mut W,
    ) -> Result<usize, vtansi::EncodeError> {
        let mut counter = 0usize;
        for (i, cap) in self.0.iter().enumerate() {
            if i > 0 {
                counter += vtansi::write_byte_into(sink, b';')?;
            }
            counter +=
                <TerminalCapability as vtansi::AnsiEncode>::encode_ansi_into(
                    cap, sink,
                )?;
        }

        Ok(counter)
    }
}

impl From<Vec<TerminalCapability>> for Capabilities {
    fn from(caps: Vec<TerminalCapability>) -> Self {
        Self(caps)
    }
}

impl<'a> vtansi::TryFromAnsi<'a> for Capabilities {
    fn try_from_ansi(bytes: &'a [u8]) -> Result<Self, vtansi::ParseError> {
        // Parse semicolon-separated capabilities
        let caps: Vec<TerminalCapability> = bytes
            .split(|b| *b == b';')
            .map(<TerminalCapability as vtansi::TryFromAnsi>::try_from_ansi)
            .collect::<Result<_, _>>()?;
        Ok(Self(caps))
    }
}

impl<'a> vtansi::TryFromAnsiIter<'a> for Capabilities {
    fn try_from_ansi_iter<I>(iter: &mut I) -> Result<Self, vtansi::ParseError>
    where
        I: Iterator<Item = &'a [u8]>,
    {
        // Consume all remaining parameters as capabilities
        let caps: Vec<TerminalCapability> = iter
            .map(<TerminalCapability as vtansi::TryFromAnsi>::try_from_ansi)
            .collect::<Result<_, _>>()?;
        Ok(Self(caps))
    }
}

/// Response to primary device attributes request (`DA1`).
///
/// *Sequence*: `CSI ? Ps ; Ps ; ... c`
///
/// Send terminal capabilities in response to a DA1 query.
///
/// The response format is `CSI ? [level] ; [cap1] ; [cap2] ; ... c`.
///
/// See <https://terminalguide.namepad.de/seq/csi_sc/> for terminal
/// support specifics.
#[derive(
    Debug, PartialOrd, PartialEq, Eq, Clone, Hash, vtansi::derive::AnsiInput,
)]
#[vtansi(csi, private = '?', finalbyte = 'c')]
pub struct PrimaryDeviceAttributesResponse {
    /// Conformance level (VT100, VT220, etc.).
    pub conformance_level: ConformanceLevel,
    /// Terminal capabilities to report.
    #[vtansi(flatten)]
    pub capabilities: Capabilities,
}

/// Response to secondary device attributes request (`DA2`).
///
/// *Sequence*: `CSI > Ps ; Ps ; Ps c`
///
/// Send terminal type and version information in response to a DA2
/// query.
///
/// The response format is `CSI > [terminal_type] ; [version] ; [extra] c`.
///
/// Common terminal type codes:
/// - 0: VT100
/// - 1: VT220
/// - 2: VT240
/// - 18: VT330
/// - 19: VT340
/// - 24: VT320
/// - 41: VT420
/// - 61: VT510
/// - 64: VT520
/// - 65: VTE-based (e.g., GNOME Terminal)
///
/// The version field typically contains the terminal version or patch
/// level.
///
/// See <https://terminalguide.namepad.de/seq/csi_sc__q/> for terminal support
/// specifics.
#[derive(
    Debug,
    PartialOrd,
    PartialEq,
    Eq,
    Clone,
    Copy,
    Hash,
    vtansi::derive::AnsiInput,
)]
#[vtansi(csi, private = '>', finalbyte = 'c')]
pub struct SecondaryDeviceAttributesResponse {
    pub terminal_type: u16,
    pub version: u16,
    pub extra: Option<u16>,
}

/// Response to tertiary device attributes request (`DECRPTUI`).
///
/// *Sequence*: `DCS ! | Pt ST`
///
/// Sent in response to a DA3 query, e.g [`RequestTerminalUnitId`].
///
/// The response format is `DCS ! | [hex_string] ST` where `hex_string`
/// is a string encoded as hexadecimal pairs.
///
/// This is less commonly supported than DA1 and DA2. When supported,
/// the unit ID is typically a string identifying the terminal
/// hardware or implementation.
///
/// # Examples
///
/// Different terminals return different unit IDs encoded as hexadecimal:
///
/// - xterm (v336+): `DCS ! | 00000000 ST`
/// - VTE (GNOME Terminal): `DCS ! | 7E565445 ST` ("~VTE")
/// - Konsole: `DCS ! | 7E4B4445 ST` ("~KDE")
/// - iTerm2: `DCS ! | 6954726D ST` ("iTrm")
///
/// See <https://terminalguide.namepad.de/seq/csi_sc__r/> for terminal
/// support specifics.
#[derive(
    Debug, PartialOrd, PartialEq, Eq, Clone, Hash, vtansi::derive::AnsiInput,
)]
#[vtansi(dcs, intermediate = "!", finalbyte = '|')]
pub struct TertiaryDeviceAttributesResponse {
    /// The terminal's unit ID (hex-decoded).
    #[vtansi(locate = "data")]
    pub data: HexString,
}

/// Select VT-XXX Conformance Level (`DECSCL`).
///
/// *Sequence*: `CSI Ps ; Ps " p`
///
/// Set the conformance level and encoding for C1 controls in terminal
/// replies.
///
/// If `level` < 61 or higher than the configured maximum this sequence
/// does nothing.
///
/// Otherwise `level` - 60 is the VT-xxx conformance level to activate
/// (i.e. `level` = 64 -> VT-4xx conformance).
///
/// If `level` > 61, the parameter `c1_encoding` is used to set the
/// encoding for C1 controls. If `c1_encoding` = 1 then use 7-bit
/// controls. If `c1_encoding` is 0 or 2 then use 8-bit controls. If
/// `c1_encoding` is explicitly set to any other value the encoding is
/// not changed.
///
/// See <https://terminalguide.namepad.de/seq/csi_sp_t_quote/> for
/// terminal support specifics.
#[derive(
    Debug,
    PartialOrd,
    PartialEq,
    Eq,
    Clone,
    Copy,
    Hash,
    vtansi::derive::AnsiOutput,
)]
#[vtansi(csi, intermediate = "\"", finalbyte = 'p')]
pub struct SelectVTConformanceLevel {
    pub level: u16,
    pub c1_encoding: Option<u8>,
}

/// Request VT-xxx Conformance Level and C1 Encoding.
///
/// *Sequence*: `DCS $ q " p ST`
///
/// Query state settable with select vt-xxx conformance level.
///
/// The terminal replies with:
///
/// `DCS $ r level ; c1_encoding ST`
///
/// Where `level` is the vt level plus 60 (i.e. 64 for vt level 4) and
/// `c1_encoding` is set to 1 if 7bit encoding of C1 controls is
/// selected.
///
/// See <https://terminalguide.namepad.de/seq/dcs-dollar-q-quote-p/> for
/// terminal support specifics.
#[derive(
    Debug,
    PartialOrd,
    PartialEq,
    Eq,
    Clone,
    Copy,
    Hash,
    vtansi::derive::AnsiOutput,
)]
#[vtansi(dcs, intermediate = "$", finalbyte = 'q', data = "\"p")]
pub struct RequestVTConformanceLevel;

/// Hex-encoded string.
///
/// Encodes a string as hex pairs (each ASCII byte becomes two hex digits).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct HexString(pub Vec<u8>);

impl HexString {
    /// Create a new `HexString` from raw bytes.
    #[must_use]
    pub const fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    /// Create a `HexString` from a string slice.
    #[must_use]
    pub fn from_string(s: &str) -> Self {
        Self(s.as_bytes().to_vec())
    }

    /// Get the decoded bytes as a string slice if valid UTF-8.
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        std::str::from_utf8(&self.0).ok()
    }

    /// Get the raw decoded bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl std::ops::Deref for HexString {
    type Target = Vec<u8>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<&str> for HexString {
    fn from(s: &str) -> Self {
        Self::from_string(s)
    }
}

impl From<String> for HexString {
    fn from(s: String) -> Self {
        Self(s.into_bytes())
    }
}

impl From<Vec<u8>> for HexString {
    fn from(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }
}

impl vtansi::AnsiEncode for HexString {
    fn encode_ansi_into<W: std::io::Write + ?Sized>(
        &self,
        sink: &mut W,
    ) -> Result<usize, vtansi::EncodeError> {
        const HEX: &[u8; 16] = b"0123456789ABCDEF";
        let mut hex = Vec::with_capacity(self.0.len() * 2);

        for &b in &self.0 {
            hex.push(HEX[(b >> 4) as usize]);
            hex.push(HEX[(b & 0x0F) as usize]);
        }

        <[u8] as vtansi::AnsiEncode>::encode_ansi_into(&hex, sink)
    }
}

impl<'a> vtansi::TryFromAnsi<'a> for HexString {
    fn try_from_ansi(bytes: &'a [u8]) -> Result<Self, vtansi::ParseError> {
        let mut result = Vec::with_capacity(bytes.len() / 2);

        for chunk in bytes.chunks(2) {
            if chunk.len() == 2
                && let Ok(s) = std::str::from_utf8(chunk)
                && let Ok(b) = u8::from_str_radix(s, 16)
            {
                result.push(b);
            }
        }

        Ok(Self(result))
    }
}

/// Wrapper for encoding multiple hex-encoded query strings.
///
/// Used internally by [`RequestTermcap`] to encode the query
/// data as semicolon-separated hex strings.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct HexStringList(Vec<HexString>);

impl std::ops::Deref for HexStringList {
    type Target = Vec<HexString>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl vtansi::AnsiEncode for HexStringList {
    fn encode_ansi_into<W: std::io::Write + ?Sized>(
        &self,
        sink: &mut W,
    ) -> Result<usize, vtansi::EncodeError> {
        let mut written = 0;

        for (i, query) in self.0.iter().enumerate() {
            if i > 0 {
                written +=
                    <[u8] as vtansi::AnsiEncode>::encode_ansi_into(b";", sink)?;
            }
            written += vtansi::AnsiEncode::encode_ansi_into(query, sink)?;
        }

        Ok(written)
    }
}

impl<'a> vtansi::TryFromAnsi<'a> for HexStringList {
    fn try_from_ansi(bytes: &'a [u8]) -> Result<Self, vtansi::ParseError> {
        let mut result = Vec::new();
        for part in bytes.split(|&b| b == b';') {
            if !part.is_empty() {
                result.push(HexString::try_from_ansi(part)?);
            }
        }
        Ok(Self(result))
    }
}

/// Request termcap/terminfo capability (`XTGETTCAP`).
///
/// *Sequence*: `DCS + q Pt ST`
///
/// Query keyboard mapping or miscellaneous terminal information using
/// the xterm termcap query mechanism.
///
/// The `queries` field contains one or more capability names to query,
/// which will be hex-encoded in the DCS sequence. Multiple queries
/// are separated by semicolons.
///
/// Common query values (before hex encoding):
/// - `colors` or `Co` - number of palette colors (256, 88, or 16)
/// - `RGB` - significant bits for direct color display
/// - `name` or `TN` - name of terminal description (e.g., "xterm")
/// - terminfo key names (e.g., `kf1` for function key 1)
///
/// The terminal replies with [`TermcapQueryResponse`].
///
/// **Note:** xterm aborts processing at the first unsuccessful query;
/// all subsequent query parts are ignored.
///
/// See <https://terminalguide.namepad.de/seq/dcs-plus-q/> for terminal
/// support specifics.
#[derive(Debug, Clone, PartialEq, Eq, Hash, vtansi::derive::AnsiOutput)]
#[vtansi(dcs, intermediate = "+", finalbyte = 'q')]
pub struct RequestTermcap {
    /// The capability names to query (will be hex-encoded).
    #[vtansi(locate = "data")]
    pub queries: HexStringList,
}

impl RequestTermcap {
    /// Create a new termcap query request with a single query.
    #[must_use]
    pub fn new(query: impl Into<HexString>) -> Self {
        Self {
            queries: HexStringList(vec![query.into()]),
        }
    }

    /// Create a new termcap query request with multiple queries.
    #[must_use]
    pub fn with_queries(
        queries: impl IntoIterator<Item = impl Into<HexString>>,
    ) -> Self {
        Self {
            queries: HexStringList(
                queries.into_iter().map(Into::into).collect(),
            ),
        }
    }
}

/// A single termcap query result with key and optional value.
///
/// When the terminal successfully resolves a query, both `key` and
/// `value` are present. When the query is valid but has no data,
/// only `key` is present with `value` set to `None`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TermcapQueryResult {
    /// The queried capability name (hex-decoded).
    pub key: HexString,
    /// The result value (hex-decoded), if available.
    pub value: Option<HexString>,
}

impl TermcapQueryResult {
    /// Check if this result has a value.
    ///
    /// Returns `true` if the query was successful and returned data,
    /// `false` if the query was valid but no data was available.
    #[must_use]
    pub fn has_value(&self) -> bool {
        self.value.is_some()
    }

    /// Get the key as a string, if it's valid UTF-8.
    #[must_use]
    pub fn key_as_str(&self) -> Option<&str> {
        self.key.as_str()
    }

    /// Get the value as a string, if present and valid UTF-8.
    #[must_use]
    pub fn value_as_str(&self) -> Option<&str> {
        self.value.as_ref().and_then(|v| v.as_str())
    }

    /// Get the value as bytes, if present.
    #[must_use]
    pub fn value_as_bytes(&self) -> Option<&[u8]> {
        self.value.as_ref().map(HexString::as_bytes)
    }
}

/// Wrapper for parsing termcap query response data.
///
/// Parses the semicolon-separated hex-encoded key=value pairs from
/// the DCS response.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct TermcapQueryResultList(Vec<TermcapQueryResult>);

impl std::ops::Deref for TermcapQueryResultList {
    type Target = Vec<TermcapQueryResult>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl vtansi::AnsiEncode for TermcapQueryResultList {
    fn encode_ansi_into<W: std::io::Write + ?Sized>(
        &self,
        sink: &mut W,
    ) -> Result<usize, vtansi::EncodeError> {
        let mut written = 0;

        for (i, result) in self.0.iter().enumerate() {
            if i > 0 {
                written +=
                    <[u8] as vtansi::AnsiEncode>::encode_ansi_into(b";", sink)?;
            }
            written += vtansi::AnsiEncode::encode_ansi_into(&result.key, sink)?;
            if let Some(ref value) = result.value {
                written +=
                    <[u8] as vtansi::AnsiEncode>::encode_ansi_into(b"=", sink)?;
                written += vtansi::AnsiEncode::encode_ansi_into(value, sink)?;
            }
        }

        Ok(written)
    }
}

impl<'a> vtansi::TryFromAnsi<'a> for TermcapQueryResultList {
    fn try_from_ansi(bytes: &'a [u8]) -> Result<Self, vtansi::ParseError> {
        if bytes.is_empty() {
            return Ok(Self(Vec::new()));
        }

        let mut results = Vec::new();
        for part in bytes.split(|&b| b == b';') {
            if part.is_empty() {
                continue;
            }

            // Split on '=' to separate key and value
            if let Some(eq_pos) = part.iter().position(|&b| b == b'=') {
                let key = HexString::try_from_ansi(&part[..eq_pos])?;
                let value = HexString::try_from_ansi(&part[eq_pos + 1..])?;
                results.push(TermcapQueryResult {
                    key,
                    value: Some(value),
                });
            } else {
                // No '=' means valid query but no data
                let key = HexString::try_from_ansi(part)?;
                results.push(TermcapQueryResult { key, value: None });
            }
        }

        Ok(Self(results))
    }
}

/// Response to termcap/terminfo capability query (`XTGETTCAP`).
///
/// *Sequence*: `DCS Ps + r Pt ST`
///
/// Response from the terminal to [`RequestTermcap`].
///
/// The response format depends on whether the query was successful:
///
/// - **Negative response** (invalid/unrecognized query): `DCS 0 + r ST`
///   - The `valid` field is `false`, and `results` is empty.
/// - **Positive response** with results: `DCS 1 + r <query>=<result>[;<query>=<result>...] ST`
///   - The `valid` field is `true`, and each result has both key and value.
/// - **Positive response** without data: `DCS 1 + r <query>[;<query>...] ST`
///   - The `valid` field is `true`, but result entries have `value` set to `None`.
///
/// Both `query` and `result` are hex-encoded strings.
///
/// Note: xterm aborts processing with the first query that is not successful,
/// and all further query parts are ignored, resulting in a negative response.
///
/// See <https://terminalguide.namepad.de/seq/dcs-plus-q/> for terminal
/// support specifics.
#[derive(Debug, Clone, PartialEq, Eq, Hash, vtansi::derive::AnsiInput)]
#[vtansi(dcs, intermediate = "+", finalbyte = 'r')]
pub struct TermcapQueryResponse {
    /// Whether the query was valid (`true` = `1`, `false` = `0`).
    #[vtansi(locate = "params")]
    pub valid: bool,
    /// The query results, empty if the query was invalid.
    #[vtansi(locate = "data")]
    pub results: TermcapQueryResultList,
}

impl TermcapQueryResponse {
    /// Create an invalid/negative response (`DCS 0 + r ST`).
    ///
    /// Use this when the query was unrecognized or invalid.
    #[must_use]
    pub fn invalid() -> Self {
        Self {
            valid: false,
            results: TermcapQueryResultList(Vec::new()),
        }
    }

    /// Create a valid response with no results.
    ///
    /// This represents a positive response where the query was recognized
    /// but no data is included in the response.
    #[must_use]
    pub fn new() -> Self {
        Self {
            valid: true,
            results: TermcapQueryResultList(Vec::new()),
        }
    }

    /// Create a valid response with the given results.
    ///
    /// Use this when the query was successful and returned data.
    #[must_use]
    pub fn with_results(results: Vec<TermcapQueryResult>) -> Self {
        Self {
            valid: true,
            results: TermcapQueryResultList(results),
        }
    }

    /// Check if the response indicates a valid query (positive response).
    ///
    /// Returns `true` if the terminal recognized the query (even if no data
    /// was available for the query).
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.valid
    }

    /// Check if this is a negative response.
    ///
    /// A negative response (`DCS 0 + r ST`) indicates that the query was
    /// invalid or unrecognized by the terminal. In this case, `results`
    /// will be empty.
    ///
    /// This is the inverse of [`is_valid`](Self::is_valid).
    #[must_use]
    pub fn is_negative(&self) -> bool {
        !self.valid
    }

    /// Check if this response has any results.
    ///
    /// Returns `false` for negative responses and for positive responses
    /// that contain no data.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.results.0.is_empty()
    }

    /// Get the first result's value as a string, if present.
    #[must_use]
    pub fn first_value_as_str(&self) -> Option<&str> {
        self.results
            .0
            .first()
            .and_then(|r| r.value.as_ref())
            .and_then(|v| v.as_str())
    }

    /// Find a result by key name.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&TermcapQueryResult> {
        self.results.0.iter().find(|r| r.key.as_str() == Some(key))
    }

    /// Get a result's value as a string by key name.
    ///
    /// Returns `None` if the key is not found or if the value is not
    /// present or not valid UTF-8.
    #[must_use]
    pub fn get_value_as_str(&self, key: &str) -> Option<&str> {
        self.get(key).and_then(|r| r.value_as_str())
    }
}

impl Default for TermcapQueryResponse {
    fn default() -> Self {
        Self::new()
    }
}

/// Request terminal name and version (`XTVERSION`).
///
/// *Sequence*: `CSI > 0 q`
///
/// Query the terminal's name and version string using the xterm
/// XTVERSION extension.
///
/// The terminal replies with [`TerminalNameAndVersionResponse`].
///
/// # Example Response Strings
///
/// Different terminals return different version strings:
/// - xterm: `XTerm(388)`
/// - VTE (GNOME Terminal): `VTE(0.74.2)`
/// - foot: `foot(1.16.2)`
/// - kitty: `kitty(0.32.2)`
/// - Alacritty: `alacritty(0.13.1)`
/// - iTerm2: `iTerm2 3.5.0`
/// - `WezTerm`: `wezterm 20240203-110809-5046fc22`
///
/// See <https://invisible-island.net/xterm/ctlseqs/ctlseqs.html> for
/// the xterm specification.
#[derive(
    Debug,
    PartialOrd,
    PartialEq,
    Eq,
    Clone,
    Copy,
    Hash,
    vtansi::derive::AnsiOutput,
)]
#[vtansi(csi, private = '>', params = ["0"], finalbyte = 'q')]
pub struct RequestTerminalNameAndVersion;

/// Response to terminal name and version request (`XTVERSION`).
///
/// *Sequence*: `DCS > | Pt ST`
///
/// Response from the terminal to [`RequestTerminalNameAndVersion`].
///
/// The response format is `DCS > | <version-string> ST` where
/// `version-string` contains the terminal name and version.
///
/// # Example Response Strings
///
/// Different terminals return different version strings:
/// - xterm: `XTerm(388)`
/// - VTE (GNOME Terminal): `VTE(0.74.2)`
/// - foot: `foot(1.16.2)`
/// - kitty: `kitty(0.32.2)`
/// - Alacritty: `alacritty(0.13.1)`
/// - iTerm2: `iTerm2 3.5.0`
/// - `WezTerm`: `wezterm 20240203-110809-5046fc22`
///
/// See <https://invisible-island.net/xterm/ctlseqs/ctlseqs.html> for
/// the xterm specification.
#[derive(Debug, Clone, PartialEq, Eq, Hash, vtansi::derive::AnsiInput)]
#[vtansi(dcs, intermediate = ">", finalbyte = '|')]
pub struct TerminalNameAndVersionResponse<'a> {
    /// The terminal name and version string.
    #[vtansi(locate = "data")]
    pub version: &'a str,
}

/// Owned version of [`TerminalNameAndVersionResponse`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TerminalNameAndVersionResponseOwned {
    /// The terminal name and version string.
    pub version: String,
}

impl<'a> From<TerminalNameAndVersionResponse<'a>>
    for TerminalNameAndVersionResponseOwned
{
    fn from(response: TerminalNameAndVersionResponse<'a>) -> Self {
        Self {
            version: response.version.to_string(),
        }
    }
}

impl TerminalNameAndVersionResponseOwned {
    /// Borrow this owned response as a borrowed version.
    #[must_use]
    pub fn borrow(&self) -> TerminalNameAndVersionResponse<'_> {
        TerminalNameAndVersionResponse {
            version: &self.version,
        }
    }
}

impl TerminalNameAndVersionResponse<'_> {
    /// Convert to an owned version.
    #[must_use]
    pub fn to_owned(&self) -> TerminalNameAndVersionResponseOwned {
        TerminalNameAndVersionResponseOwned {
            version: self.version.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vtansi::AnsiEncode;
    use vtansi::TryFromAnsi;

    #[test]
    fn test_primary_device_attributes_response_encoding() {
        let response = PrimaryDeviceAttributesResponse {
            conformance_level: ConformanceLevel::VT220,
            capabilities: Capabilities(vec![
                TerminalCapability::Columns132,
                TerminalCapability::SixelGraphics,
                TerminalCapability::Color,
            ]),
        };

        let mut buf = Vec::new();
        response.encode_ansi_into(&mut buf).unwrap();
        let encoded = String::from_utf8(buf).unwrap();

        assert_eq!(encoded, "\x1b[?62;1;4;22c");
    }

    #[test]
    fn test_secondary_device_attributes_response_encoding() {
        let response = SecondaryDeviceAttributesResponse {
            terminal_type: 65,
            version: 6800,
            extra: Some(1),
        };

        let mut buf = Vec::new();
        response.encode_ansi_into(&mut buf).unwrap();
        let encoded = String::from_utf8(buf).unwrap();

        assert_eq!(encoded, "\x1b[>65;6800;1c");
    }

    #[test]
    fn test_secondary_device_attributes_response_encoding_no_extra() {
        let response = SecondaryDeviceAttributesResponse {
            terminal_type: 1,
            version: 0,
            extra: None,
        };

        let mut buf = Vec::new();
        response.encode_ansi_into(&mut buf).unwrap();
        let encoded = String::from_utf8(buf).unwrap();

        assert_eq!(encoded, "\x1b[>1;0c");
    }

    #[test]
    fn test_tertiary_device_attributes_response_encoding() {
        let response = TertiaryDeviceAttributesResponse {
            data: HexString::from_string("~VTE"),
        };

        let mut buf = Vec::new();
        response.encode_ansi_into(&mut buf).unwrap();
        let encoded = String::from_utf8(buf).unwrap();

        assert_eq!(encoded, "\x1bP!|7E565445\x1b\\");
    }

    #[test]
    fn test_request_terminal_unit_id_encoding() {
        let request = RequestTerminalUnitId;

        let mut buf = Vec::new();
        request.encode_ansi_into(&mut buf).unwrap();
        let encoded = String::from_utf8(buf).unwrap();

        assert_eq!(encoded, "\x1b[=0c");
    }

    #[test]
    fn test_select_vt_conformance_level_encoding() {
        let cmd = SelectVTConformanceLevel {
            level: 64,
            c1_encoding: Some(1),
        };

        let mut buf = Vec::new();
        cmd.encode_ansi_into(&mut buf).unwrap();
        let encoded = String::from_utf8(buf).unwrap();

        assert_eq!(encoded, "\x1b[64;1\"p");
    }

    #[test]
    fn test_select_vt_conformance_level_encoding_no_c1() {
        let cmd = SelectVTConformanceLevel {
            level: 62,
            c1_encoding: None,
        };

        let mut buf = Vec::new();
        cmd.encode_ansi_into(&mut buf).unwrap();
        let encoded = String::from_utf8(buf).unwrap();

        assert_eq!(encoded, "\x1b[62\"p");
    }

    #[test]
    fn test_request_termcap_query_single() {
        let request = RequestTermcap::new("colors");

        let mut buf = Vec::new();
        request.encode_ansi_into(&mut buf).unwrap();
        let encoded = String::from_utf8(buf).unwrap();

        // "colors" hex-encoded = "636F6C6F7273"
        assert_eq!(encoded, "\x1bP+q636F6C6F7273\x1b\\");
    }

    #[test]
    fn test_request_termcap_query_multiple() {
        let request = RequestTermcap::with_queries(["colors", "RGB"]);

        let mut buf = Vec::new();
        request.encode_ansi_into(&mut buf).unwrap();
        let encoded = String::from_utf8(buf).unwrap();

        // "colors" = "636F6C6F7273", "RGB" = "524742"
        assert_eq!(encoded, "\x1bP+q636F6C6F7273;524742\x1b\\");
    }

    #[test]
    fn test_hex_string_roundtrip() {
        let original = HexString::from_string("test");

        let mut buf = Vec::new();
        original.encode_ansi_into(&mut buf).unwrap();

        // Should encode to "74657374"
        assert_eq!(&buf, b"74657374");

        let decoded = HexString::try_from_ansi(&buf).unwrap();
        assert_eq!(decoded.as_str(), Some("test"));
    }

    #[test]
    fn test_request_terminal_name_and_version_encoding() {
        let request = RequestTerminalNameAndVersion;

        let mut buf = Vec::new();
        request.encode_ansi_into(&mut buf).unwrap();
        let encoded = String::from_utf8(buf).unwrap();

        assert_eq!(encoded, "\x1b[>0q");
    }

    #[test]
    fn test_termcap_query_result_list_negative_response() {
        // Negative response has empty data
        let result = TermcapQueryResultList::try_from_ansi(b"").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_termcap_query_result_list_valid_with_value() {
        // "colors" = 636F6C6F7273, "256" = 323536
        let result =
            TermcapQueryResultList::try_from_ansi(b"636F6C6F7273=323536")
                .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].key.as_str(), Some("colors"));
        assert_eq!(result[0].value_as_str(), Some("256"));
        assert!(result[0].has_value());
    }

    #[test]
    fn test_termcap_query_result_list_valid_without_value() {
        // Valid query but no data available - just key, no "="
        // "colors" = 636F6C6F7273
        let result =
            TermcapQueryResultList::try_from_ansi(b"636F6C6F7273").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].key.as_str(), Some("colors"));
        assert!(result[0].value.is_none());
        assert!(!result[0].has_value());
    }

    #[test]
    fn test_termcap_query_result_list_multiple() {
        // "colors" = 636F6C6F7273, "256" = 323536
        // "RGB" = 524742, "8" = 38
        let result = TermcapQueryResultList::try_from_ansi(
            b"636F6C6F7273=323536;524742=38",
        )
        .unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].key.as_str(), Some("colors"));
        assert_eq!(result[0].value_as_str(), Some("256"));
        assert_eq!(result[1].key.as_str(), Some("RGB"));
        assert_eq!(result[1].value_as_str(), Some("8"));
    }

    #[test]
    fn test_termcap_query_response_helpers() {
        let response =
            TermcapQueryResponse::with_results(vec![TermcapQueryResult {
                key: HexString::from_string("colors"),
                value: Some(HexString::from_string("256")),
            }]);

        assert!(response.is_valid());
        assert!(!response.is_negative());
        assert!(!response.is_empty());
        assert_eq!(response.first_value_as_str(), Some("256"));
        assert_eq!(response.get_value_as_str("colors"), Some("256"));
        assert!(response.get("unknown").is_none());
    }

    #[test]
    fn test_termcap_query_response_negative() {
        let response = TermcapQueryResponse::invalid();

        assert!(!response.is_valid());
        assert!(response.is_negative());
        assert!(response.is_empty());
        assert!(response.first_value_as_str().is_none());
    }

    #[test]
    fn test_termcap_query_response_constructors() {
        // Test invalid() constructor
        let invalid = TermcapQueryResponse::invalid();
        assert!(!invalid.valid);
        assert!(invalid.results.is_empty());

        // Test new() constructor - valid with no results
        let empty_valid = TermcapQueryResponse::new();
        assert!(empty_valid.valid);
        assert!(empty_valid.results.is_empty());

        // Test with_results() constructor
        let with_data = TermcapQueryResponse::with_results(vec![
            TermcapQueryResult {
                key: HexString::from_string("colors"),
                value: Some(HexString::from_string("256")),
            },
            TermcapQueryResult {
                key: HexString::from_string("RGB"),
                value: None,
            },
        ]);
        assert!(with_data.valid);
        assert_eq!(with_data.results.len(), 2);
        assert_eq!(with_data.get_value_as_str("colors"), Some("256"));
        assert!(with_data.get("RGB").is_some());
        assert!(with_data.get("RGB").unwrap().value.is_none());
    }
}
