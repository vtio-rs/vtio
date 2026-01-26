//! Window control commands.

use vtansi::bitflags;

/// Title stack target.
///
/// Specifies which title(s) to push or pop from the stack.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    vtansi::derive::ToAnsi,
    vtansi::derive::FromAnsi,
    num_enum::TryFromPrimitive,
    num_enum::IntoPrimitive,
)]
#[repr(u8)]
pub enum TitleStackTarget {
    /// Icon name and window title.
    Both = 0,
    /// Icon name only.
    IconName = 1,
    /// Window title only.
    WindowTitle = 2,
    /// Other unsupported value.
    #[num_enum(catch_all)]
    Other(u8),
}

/// Maximization mode.
///
/// Specifies how to maximize the terminal window.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    vtansi::derive::ToAnsi,
    vtansi::derive::FromAnsi,
    num_enum::TryFromPrimitive,
    num_enum::IntoPrimitive,
)]
#[repr(u8)]
pub enum MaximizeMode {
    /// Restore (un-maximize) the window.
    Restore = 0,
    /// Maximize the window.
    Maximize = 1,
    /// Maximize vertically only.
    MaximizeVertically = 2,
    /// Maximize horizontally only.
    MaximizeHorizontally = 3,
    /// Other unsupported value.
    #[num_enum(catch_all)]
    Other(u8),
}

/// Set terminal window title and icon name.
///
/// *Sequence*: `OSC 0 ; Pt ST`
///
/// Set both the window title and icon name to the same string.
///
/// See <https://terminalguide.namepad.de/seq/osc-0/> for
/// terminal support specifics.
#[derive(Debug, Clone, PartialEq, Eq, Hash, vtansi::derive::AnsiOutput)]
#[vtansi(osc, number = "0")]
pub struct SetTitleAndIconName<'a> {
    pub title: &'a str,
}

impl SetTitleAndIconName<'_> {
    /// Convert to an owned version.
    #[must_use]
    pub fn to_owned(&self) -> SetTitleAndIconNameOwned {
        SetTitleAndIconNameOwned {
            title: self.title.to_string(),
        }
    }
}

/// Owned version of [`SetTitleAndIconName`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SetTitleAndIconNameOwned {
    pub title: String,
}

impl<'a> From<SetTitleAndIconName<'a>> for SetTitleAndIconNameOwned {
    fn from(value: SetTitleAndIconName<'a>) -> Self {
        Self {
            title: value.title.to_string(),
        }
    }
}

impl SetTitleAndIconNameOwned {
    /// Borrow this owned struct as a borrowed version.
    #[must_use]
    pub fn borrow(&self) -> SetTitleAndIconName<'_> {
        SetTitleAndIconName { title: &self.title }
    }
}

/// Set terminal window title.
///
/// *Sequence*: `OSC 2 ; Pt ST`
///
/// Set the window title.
///
/// See <https://terminalguide.namepad.de/seq/osc-2/> for
/// terminal support specifics.
#[derive(Debug, Clone, PartialEq, Eq, Hash, vtansi::derive::AnsiOutput)]
#[vtansi(osc, number = "2")]
pub struct SetTitle<'a> {
    pub title: &'a str,
}

impl SetTitle<'_> {
    /// Convert to an owned version.
    #[must_use]
    pub fn to_owned(&self) -> SetTitleOwned {
        SetTitleOwned {
            title: self.title.to_string(),
        }
    }
}

/// Owned version of [`SetTitle`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SetTitleOwned {
    pub title: String,
}

impl<'a> From<SetTitle<'a>> for SetTitleOwned {
    fn from(value: SetTitle<'a>) -> Self {
        Self {
            title: value.title.to_string(),
        }
    }
}

impl SetTitleOwned {
    /// Borrow this owned struct as a borrowed version.
    #[must_use]
    pub fn borrow(&self) -> SetTitle<'_> {
        SetTitle { title: &self.title }
    }
}

/// Set icon name.
///
/// *Sequence*: `OSC 1 ; Pt ST`
///
/// Set the icon name for the terminal window.
///
/// See <https://terminalguide.namepad.de/seq/osc-1/> for
/// terminal support specifics.
#[derive(Debug, Clone, PartialEq, Eq, Hash, vtansi::derive::AnsiOutput)]
#[vtansi(osc, number = "1")]
pub struct SetIconName<'a> {
    pub name: &'a str,
}

impl SetIconName<'_> {
    /// Convert to an owned version.
    #[must_use]
    pub fn to_owned(&self) -> SetIconNameOwned {
        SetIconNameOwned {
            name: self.name.to_string(),
        }
    }
}

/// Owned version of [`SetIconName`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SetIconNameOwned {
    pub name: String,
}

impl<'a> From<SetIconName<'a>> for SetIconNameOwned {
    fn from(value: SetIconName<'a>) -> Self {
        Self {
            name: value.name.to_string(),
        }
    }
}

impl SetIconNameOwned {
    /// Borrow this owned struct as a borrowed version.
    #[must_use]
    pub fn borrow(&self) -> SetIconName<'_> {
        SetIconName { name: &self.name }
    }
}

/// Get terminal window title.
///
/// *Sequence*: `CSI 21 t`
///
/// Request the current window title. The terminal responds with
/// `OSC 2 ; Pt ST` or `OSC l Pt ST`.
///
/// See <https://terminalguide.namepad.de/seq/csi_st-21/> for
/// terminal support specifics.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, vtansi::derive::AnsiOutput,
)]
#[vtansi(csi, params = ["21"], finalbyte = 't')]
pub struct GetTitle;

/// Get icon name.
///
/// *Sequence*: `CSI 20 t`
///
/// Request the current icon name. The terminal responds with
/// `OSC 1 ; Pt ST` or `OSC L Pt ST`.
///
/// See <https://terminalguide.namepad.de/seq/csi_st-20/> for
/// terminal support specifics.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, vtansi::derive::AnsiOutput,
)]
#[vtansi(csi, params = ["20"], finalbyte = 't')]
pub struct GetIconName;

/// Push terminal title onto stack.
///
/// *Sequence*: `CSI 22 ; Ps t`
///
/// Push the current title onto an internal stack. The optional
/// parameter specifies which title to push.
///
/// See <https://terminalguide.namepad.de/seq/csi_st-22/> for
/// terminal support specifics.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, vtansi::derive::AnsiOutput,
)]
#[vtansi(csi, params = ["22"], finalbyte = 't')]
pub struct PushTitle {
    /// Which title to push.
    pub which: Option<TitleStackTarget>,
}

/// Pop terminal title from stack.
///
/// *Sequence*: `CSI 23 ; Ps t`
///
/// Pop a title from the internal stack and set it as the current title.
/// The optional parameter specifies which title to pop.
///
/// See <https://terminalguide.namepad.de/seq/csi_st-23/> for
/// terminal support specifics.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, vtansi::derive::AnsiOutput,
)]
#[vtansi(csi, params = ["23"], finalbyte = 't')]
pub struct PopTitle {
    /// Which title to pop.
    pub which: Option<TitleStackTarget>,
}

/// Restore terminal window.
///
/// *Sequence*: `CSI 1 t`
///
/// Restore (de-iconify) the terminal window.
///
/// See <https://terminalguide.namepad.de/seq/csi_st-1/> for
/// terminal support specifics.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, vtansi::derive::AnsiOutput,
)]
#[vtansi(csi, params = ["1"], finalbyte = 't')]
pub struct RestoreWindow;

/// Minimize terminal window.
///
/// *Sequence*: `CSI 2 t`
///
/// Minimize (iconify) the terminal window.
///
/// See <https://terminalguide.namepad.de/seq/csi_st-2/> for
/// terminal support specifics.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, vtansi::derive::AnsiOutput,
)]
#[vtansi(csi, params = ["2"], finalbyte = 't')]
pub struct MinimizeWindow;

/// Raise terminal window.
///
/// *Sequence*: `CSI 5 t`
///
/// Raise the terminal window to the front of the stacking order.
///
/// See <https://terminalguide.namepad.de/seq/csi_st-5/> for
/// terminal support specifics.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, vtansi::derive::AnsiOutput,
)]
#[vtansi(csi, params = ["5"], finalbyte = 't')]
pub struct RaiseWindow;

/// Lower terminal window.
///
/// *Sequence*: `CSI 6 t`
///
/// Lower the terminal window to the back of the stacking order.
///
/// See <https://terminalguide.namepad.de/seq/csi_st-6/> for
/// terminal support specifics.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, vtansi::derive::AnsiOutput,
)]
#[vtansi(csi, params = ["6"], finalbyte = 't')]
pub struct LowerWindow;

/// Refresh terminal window.
///
/// *Sequence*: `CSI 7 t`
///
/// Refresh (redraw) the terminal window.
///
/// See <https://terminalguide.namepad.de/seq/csi_st-7/> for
/// terminal support specifics.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, vtansi::derive::AnsiOutput,
)]
#[vtansi(csi, params = ["7"], finalbyte = 't')]
pub struct RefreshWindow;

/// Set terminal window position.
///
/// Move the terminal window to the specified position in pixels,
/// relative to the upper-left corner of the screen.
///
/// See <https://terminalguide.namepad.de/seq/csi_st-3/> for
/// terminal support specifics.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, vtansi::derive::AnsiOutput,
)]
#[vtansi(csi, params = ["3"], finalbyte = 't')]
pub struct SetWindowPosition {
    /// X coordinate in pixels.
    pub x: u16,
    /// Y coordinate in pixels.
    pub y: u16,
}

/// Set terminal window size in pixels.
///
/// Resize the terminal window to the specified size in pixels.
///
/// See <https://terminalguide.namepad.de/seq/csi_st-4/> for
/// terminal support specifics.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, vtansi::derive::AnsiOutput,
)]
#[vtansi(csi, params = ["4"], finalbyte = 't')]
pub struct SetWindowSizePixels {
    /// Height in pixels.
    pub height: u16,
    /// Width in pixels.
    pub width: u16,
}

/// Set terminal size in characters.
///
/// *Sequence*: `CSI 8 ; Ps ; Ps t`
///
/// Where the first `Ps` is the number of rows and the second `Ps` is the number of columns.
///
/// Resize the terminal window to the specified size in character cells
/// (rows and columns).
///
/// See <https://terminalguide.namepad.de/seq/csi_st-8/> for
/// terminal support specifics.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, vtansi::derive::AnsiOutput,
)]
#[vtansi(csi, params = ["8"], finalbyte = 't')]
pub struct SetSize {
    /// Number of rows.
    pub rows: u16,
    /// Number of columns.
    pub cols: u16,
}

/// Maximize terminal window.
///
/// Maximize the terminal window. The parameter specifies the
/// maximization mode.
///
/// See <https://terminalguide.namepad.de/seq/csi_st-9/> for
/// terminal support specifics.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, vtansi::derive::AnsiOutput,
)]
#[vtansi(csi, params = ["9"], finalbyte = 't')]
pub struct MaximizeWindow {
    /// Maximization mode.
    pub mode: MaximizeMode,
}

/// Maximize terminal window (alternate form).
///
/// Alternate sequence for maximizing the terminal window.
///
/// See <https://terminalguide.namepad.de/seq/csi_st-10/> for
/// terminal support specifics.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, vtansi::derive::AnsiOutput,
)]
#[vtansi(csi, params = ["10"], finalbyte = 't')]
pub struct MaximizeWindowAlt {
    /// Maximization mode.
    pub mode: MaximizeMode,
}

/// Report terminal window state.
///
/// Request whether the terminal window is iconified or not.
/// The terminal responds with `CSI 1 t` if not iconified or
/// `CSI 2 t` if iconified.
///
/// See <https://terminalguide.namepad.de/seq/csi_st-11/> for
/// terminal support specifics.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, vtansi::derive::AnsiOutput,
)]
#[vtansi(csi, params = ["11"], finalbyte = 't')]
pub struct ReportWindowState;

/// Coordinate system for window position reporting.
///
/// Specifies the coordinate system to use when reporting window position.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    vtansi::derive::ToAnsi,
    vtansi::derive::FromAnsi,
    num_enum::TryFromPrimitive,
    num_enum::IntoPrimitive,
)]
#[repr(u8)]
pub enum WindowPositionSizeReportMode {
    /// Report position or size based on window top-left corner
    /// (including window decorations).
    Outer = 0,
    /// Report position or size baed on the inner top-left corner of the
    /// window decorations.
    Inner = 2,
    /// Other unsupported value.
    #[num_enum(catch_all)]
    Other(u8),
}

/// Report terminal window position.
///
/// Request the terminal window position in pixels.
/// The terminal responds with `CSI 3 ; x ; y t`.
///
/// See <https://terminalguide.namepad.de/seq/csi_st-13/> for
/// terminal support specifics.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, vtansi::derive::AnsiOutput,
)]
#[vtansi(csi, params = ["13"], finalbyte = 't')]
pub struct ReportWindowPosition {
    /// Optional reporting mode.
    pub mode: Option<WindowPositionSizeReportMode>,
}

/// Report terminal window size in pixels.
///
/// Request the terminal window size in pixels.
/// The terminal responds with `CSI 4 ; height ; width t`.
///
/// See <https://terminalguide.namepad.de/seq/csi_st-14/> for
/// terminal support specifics.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, vtansi::derive::AnsiOutput,
)]
#[vtansi(csi, params = ["14"], finalbyte = 't')]
pub struct ReportWindowSizePixels {
    /// Optional reporting mode.
    pub mode: Option<WindowPositionSizeReportMode>,
}

/// Report screen size in pixels.
///
/// *Sequence*: `CSI 15 t`
///
/// Request the screen size in pixels. The terminal responds with
/// `CSI 5 ; Ps ; Ps t` ([`ScreenSizePixelsReport`]).
///
/// *Sequence*: `CSI 14 ; Ps t`
///
/// Request the window size in pixels. The terminal responds with
/// `CSI 4 ; Ps ; Ps t` ([`WindowSizePixelsReport`]).
///
/// Request the screen size in pixels.
/// The terminal responds with `CSI 5 ; height ; width t`.
///
/// See <https://terminalguide.namepad.de/seq/csi_st-15/> for
/// terminal support specifics.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, vtansi::derive::AnsiOutput,
)]
#[vtansi(csi, params = ["15"], finalbyte = 't')]
pub struct ReportScreenSizePixels;

/// Report cell size in pixels.
///
/// *Sequence*: `CSI 16 t`
///
/// Request the character cell size in pixels. The terminal responds with
/// `CSI 6 ; Ps ; Ps t` ([`CellSizePixelsReport`]).
///
/// Request the character cell size in pixels.
/// The terminal responds with `CSI 6 ; height ; width t`.
///
/// See <https://terminalguide.namepad.de/seq/csi_st-16/> for
/// terminal support specifics.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, vtansi::derive::AnsiOutput,
)]
#[vtansi(csi, params = ["16"], finalbyte = 't')]
pub struct ReportCellSizePixels;

/// Report screen size in characters.
///
/// *Sequence*: `CSI 19 t`
///
/// Request the screen size in characters. The terminal responds with
/// `CSI 9 ; Ps ; Ps t` ([`ScreenSizeReport`]).
///
/// *Sequence*: `CSI 18 t`
///
/// Request the terminal size in characters. The terminal responds with
/// `CSI 8 ; Ps ; Ps t` ([`SizeReport`]).
///
/// Request the terminal size in character cells (rows and columns).
/// The terminal responds with `CSI 8 ; rows ; cols t`.
///
/// See <https://terminalguide.namepad.de/seq/csi_st-18/> for
/// terminal support specifics.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, vtansi::derive::AnsiOutput,
)]
#[vtansi(csi, params = ["18"], finalbyte = 't')]
pub struct ReportSize;

/// Report screen size in character cells.
///
/// Request the screen size in character cells.
/// The terminal responds with `CSI 9 ; rows ; cols t`.
///
/// See <https://terminalguide.namepad.de/seq/csi_st-19/> for
/// terminal support specifics.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, vtansi::derive::AnsiOutput,
)]
#[vtansi(csi, params = ["19"], finalbyte = 't')]
pub struct ReportScreenSize;

/// Window state.
///
/// Indicate whether the window is iconified or not.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    vtansi::derive::ToAnsi,
    vtansi::derive::FromAnsi,
    num_enum::TryFromPrimitive,
    num_enum::IntoPrimitive,
)]
#[repr(u8)]
pub enum WindowState {
    /// Window is not iconified (normal state).
    NotIconified = 1,
    /// Window is iconified (minimized).
    Iconified = 2,
    /// Other unsupported value.
    #[num_enum(catch_all)]
    Other(u8),
}

/// Window state report.
///
/// *Sequence*: `CSI Ps t` where `Ps` is 1 (not iconified) or 2 (iconified).
///
/// Response to [`ReportWindowState`] request.
/// Report whether the window is iconified or not.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, vtansi::derive::AnsiInput,
)]
#[vtansi(csi, finalbyte = 't')]
pub struct WindowStateReport {
    pub state: WindowState,
}

/// Window position report.
///
/// *Sequence*: `CSI 3 ; Ps ; Ps t`
///
/// Response to [`ReportWindowPosition`] request.
/// Report the window position in pixels.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, vtansi::derive::AnsiInput,
)]
#[vtansi(csi, params = ["3"], finalbyte = 't')]
pub struct WindowPositionReport {
    /// X coordinate in pixels.
    pub x: u16,
    /// Y coordinate in pixels.
    pub y: u16,
}

/// Window size in pixels report.
///
/// *Sequence*: `CSI 4 ; Ps ; Ps t`
///
/// Response to [`ReportWindowSizePixels`] request.
/// Report the window size in pixels.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, vtansi::derive::AnsiInput,
)]
#[vtansi(csi, params = ["4"], finalbyte = 't')]
pub struct WindowSizePixelsReport {
    /// Height in pixels.
    pub height: u16,
    /// Width in pixels.
    pub width: u16,
}

/// Screen size in pixels report.
///
/// *Sequence*: `CSI 5 ; Ps ; Ps t`
///
/// Response to [`ReportScreenSizePixels`] request.
/// Report the screen size in pixels.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, vtansi::derive::AnsiInput,
)]
#[vtansi(csi, params = ["5"], finalbyte = 't')]
pub struct ScreenSizePixelsReport {
    /// Height in pixels.
    pub height: u16,
    /// Width in pixels.
    pub width: u16,
}

/// Cell size in pixels report.
///
/// *Sequence*: `CSI 6 ; Ps ; Ps t`
///
/// Response to [`ReportCellSizePixels`] request.
/// Report the character cell size in pixels.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, vtansi::derive::AnsiInput,
)]
#[vtansi(csi, params = ["6"], finalbyte = 't')]
pub struct CellSizePixelsReport {
    /// Cell height in pixels.
    pub height: u16,
    /// Cell width in pixels.
    pub width: u16,
}

/// Terminal size report.
///
/// Response to [`ReportSize`] request.
/// Report the terminal size in character cells (rows and columns).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, vtansi::derive::AnsiInput,
)]
#[vtansi(csi, params = ["8"], finalbyte = 't')]
pub struct SizeReport {
    /// Number of rows.
    pub rows: u16,
    /// Number of columns.
    pub cols: u16,
}

/// Screen size report.
///
/// Response to [`ReportScreenSize`] request.
/// Report the screen size in character cells.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, vtansi::derive::AnsiInput,
)]
#[vtansi(csi, params = ["9"], finalbyte = 't')]
pub struct ScreenSizeReport {
    /// Number of rows.
    pub rows: u16,
    /// Number of columns.
    pub cols: u16,
}

// ============================================================================
// Title Mode Features (XTSMTITLE / XTRMTITLE)
// ============================================================================

bitflags! {
    /// Title mode features for controlling how xterm interprets title sequences.
    ///
    /// These flags control how OSC 0, 1, and 2 title-setting sequences are
    /// interpreted and how title queries respond.
    ///
    /// See <https://invisible-island.net/xterm/ctlseqs/ctlseqs.html> for
    /// more information on xterm's title mode features.
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize), serde(transparent))]
    #[derive(Debug, PartialOrd, PartialEq, Eq, Clone, Copy, Hash, Default)]
    pub struct TitleModeFeatures: u8 {
        /// Set window/icon labels using hexadecimal encoding.
        ///
        /// When set, title strings in OSC sequences are interpreted as
        /// hexadecimal-encoded bytes.
        const SET_HEX = 0b0001;

        /// Query window/icon labels using hexadecimal encoding.
        ///
        /// When set, title query responses use hexadecimal encoding.
        const QUERY_HEX = 0b0010;

        /// Set window/icon labels using UTF-8 encoding.
        ///
        /// When set, title strings are interpreted as UTF-8.
        /// This overrides [`SET_HEX`](Self::SET_HEX) if both are set.
        const SET_UTF8 = 0b0100;

        /// Query window/icon labels using UTF-8 encoding.
        ///
        /// When set, title query responses use UTF-8 encoding.
        /// This overrides [`QUERY_HEX`](Self::QUERY_HEX) if both are set.
        const QUERY_UTF8 = 0b1000;
    }
}

/// Set title mode features (`XTSMTITLE`).
///
/// *Sequence*: `CSI > Pm t`
///
/// Set one or more title mode flags that control how xterm interprets
/// OSC 0, 1, and 2 title-setting sequences. Multiple parameters can be
/// specified to set multiple modes at once.
///
/// # Example
///
/// ```
/// use vtio::event::window::{SetTitleModeFeatures, TitleModeFeatures};
/// use vtansi::AnsiEncode;
///
/// // Enable UTF-8 encoding for setting titles
/// let cmd = SetTitleModeFeatures(TitleModeFeatures::SET_UTF8);
/// let mut buf = Vec::new();
/// cmd.encode_ansi_into(&mut buf).unwrap();
/// assert_eq!(buf, b"\x1b[>4t");
/// ```
///
/// See <https://invisible-island.net/xterm/ctlseqs/ctlseqs.html> for
/// more information.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, vtansi::derive::AnsiOutput,
)]
#[vtansi(csi, private = '>', finalbyte = 't')]
pub struct SetTitleModeFeatures(pub TitleModeFeatures);

/// Reset title mode features (`XTRMTITLE`).
///
/// *Sequence*: `CSI > Pm T`
///
/// Reset one or more title mode flags that control how xterm interprets
/// OSC 0, 1, and 2 title-setting sequences. Multiple parameters can be
/// specified to reset multiple modes at once.
///
/// # Example
///
/// ```
/// use vtio::event::window::{ResetTitleModeFeatures, TitleModeFeatures};
/// use vtansi::AnsiEncode;
///
/// // Disable UTF-8 encoding for setting titles
/// let cmd = ResetTitleModeFeatures(TitleModeFeatures::SET_UTF8);
/// let mut buf = Vec::new();
/// cmd.encode_ansi_into(&mut buf).unwrap();
/// assert_eq!(buf, b"\x1b[>4T");
/// ```
///
/// See <https://invisible-island.net/xterm/ctlseqs/ctlseqs.html> for
/// more information.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, vtansi::derive::AnsiOutput,
)]
#[vtansi(csi, private = '>', finalbyte = 'T')]
pub struct ResetTitleModeFeatures(pub TitleModeFeatures);

/// Report title stack position (`XTTITLEPOS`).
///
/// *Sequence*: `CSI # S`
///
/// Request the current position on the title stack. The terminal responds
/// with [`TitleStackPositionReport`].
///
/// The title stack is used by [`PushTitle`] and [`PopTitle`] to save and
/// restore window titles.
///
/// # Example
///
/// ```
/// use vtio::event::window::ReportTitleStackPosition;
/// use vtansi::StaticAnsiEncode;
///
/// assert_eq!(ReportTitleStackPosition::BYTES, b"\x1b[#S");
/// ```
///
/// See <https://invisible-island.net/xterm/ctlseqs/ctlseqs.html> for
/// more information.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, vtansi::derive::AnsiOutput,
)]
#[vtansi(csi, intermediate = "#", finalbyte = 'S')]
pub struct ReportTitleStackPosition;

/// Title stack position report.
///
/// *Sequence*: `CSI Ps # S`
///
/// Response to [`ReportTitleStackPosition`] request.
///
/// The position value indicates:
/// - `0`: The stack is empty
/// - `1-10`: Current position on the stack (1 is the first pushed entry)
///
/// # Example
///
/// ```
/// use vtio::event::window::TitleStackPositionReport;
/// use vtansi::AnsiEncode;
///
/// // Create a report indicating position 3 on the stack
/// let report = TitleStackPositionReport { position: 3 };
/// let mut buf = Vec::new();
/// report.encode_ansi_into(&mut buf).unwrap();
/// assert_eq!(buf, b"\x1b[3#S");
/// ```
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, vtansi::derive::AnsiInput,
)]
#[vtansi(csi, intermediate = "#", finalbyte = 'S')]
pub struct TitleStackPositionReport {
    /// Current position on the title stack.
    ///
    /// - `0`: The stack is empty
    /// - `1-10`: Current position on the stack
    pub position: u8,
}

#[cfg(test)]
mod tests {
    use super::*;
    use vtansi::{AnsiEncode, StaticAnsiEncode};

    #[test]
    fn test_set_title_and_icon_name() {
        let cmd = SetTitleAndIconName {
            title: "Test Title",
        };
        let mut buf = Vec::new();
        cmd.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "\x1b]0;Test Title\x1b\\");
    }

    #[test]
    fn test_set_title() {
        let cmd = SetTitle {
            title: "Window Title",
        };
        let mut buf = Vec::new();
        cmd.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(
            String::from_utf8(buf).unwrap(),
            "\x1b]2;Window Title\x1b\\"
        );
    }

    #[test]
    fn test_set_icon_name() {
        let cmd = SetIconName { name: "Icon" };
        let mut buf = Vec::new();
        cmd.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "\x1b]1;Icon\x1b\\");
    }

    #[test]
    fn test_get_title() {
        assert_eq!(GetTitle::BYTES, b"\x1b[21t");
    }

    #[test]
    fn test_get_icon_name() {
        assert_eq!(GetIconName::BYTES, b"\x1b[20t");
    }

    #[test]
    fn test_push_title_without_which() {
        let cmd = PushTitle { which: None };
        let mut buf = Vec::new();
        cmd.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1b[22t");
    }

    #[test]
    fn test_push_title_with_which() {
        let cmd = PushTitle {
            which: Some(TitleStackTarget::WindowTitle),
        };
        let mut buf = Vec::new();
        cmd.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "\x1b[22;2t");
    }

    #[test]
    fn test_pop_title_without_which() {
        let cmd = PopTitle { which: None };
        let mut buf = Vec::new();
        cmd.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "\x1b[23t");
    }

    #[test]
    fn test_pop_title_with_which() {
        let cmd = PopTitle {
            which: Some(TitleStackTarget::IconName),
        };
        let mut buf = Vec::new();
        cmd.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "\x1b[23;1t");
    }

    #[test]
    fn test_restore_window() {
        assert_eq!(RestoreWindow::BYTES, b"\x1b[1t");
    }

    #[test]
    fn test_minimize_window() {
        assert_eq!(MinimizeWindow::BYTES, b"\x1b[2t");
    }

    #[test]
    fn test_raise_window() {
        assert_eq!(RaiseWindow::BYTES, b"\x1b[5t");
    }

    #[test]
    fn test_lower_window() {
        assert_eq!(LowerWindow::BYTES, b"\x1b[6t");
    }

    #[test]
    fn test_refresh_window() {
        assert_eq!(RefreshWindow::BYTES, b"\x1b[7t");
    }

    #[test]
    fn test_set_window_position() {
        let cmd = SetWindowPosition { x: 100, y: 200 };
        let mut buf = Vec::new();
        cmd.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "\x1b[3;100;200t");
    }

    #[test]
    fn test_set_window_size_pixels() {
        let cmd = SetWindowSizePixels {
            height: 600,
            width: 800,
        };
        let mut buf = Vec::new();
        cmd.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "\x1b[4;600;800t");
    }

    #[test]
    fn test_set_size() {
        let cmd = SetSize { rows: 24, cols: 80 };
        let mut buf = Vec::new();
        cmd.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "\x1b[8;24;80t");
    }

    #[test]
    fn test_maximize_window() {
        let cmd = MaximizeWindow {
            mode: MaximizeMode::Maximize,
        };
        let mut buf = Vec::new();
        cmd.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "\x1b[9;1t");
    }

    #[test]
    fn test_maximize_window_alt() {
        let cmd = MaximizeWindowAlt {
            mode: MaximizeMode::MaximizeVertically,
        };
        let mut buf = Vec::new();
        cmd.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "\x1b[10;2t");
    }

    #[test]
    fn test_report_window_state() {
        assert_eq!(ReportWindowState::BYTES, b"\x1b[11t");
    }

    #[test]
    fn test_report_window_position_without_mode() {
        let cmd = ReportWindowPosition { mode: None };
        let mut buf = Vec::new();
        cmd.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "\x1b[13t");
    }

    #[test]
    fn test_report_window_position_with_mode() {
        let cmd = ReportWindowPosition {
            mode: Some(WindowPositionSizeReportMode::Inner),
        };
        let mut buf = Vec::new();
        cmd.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "\x1b[13;2t");
    }

    #[test]
    fn test_report_window_size_pixels() {
        let cmd = ReportWindowSizePixels {
            mode: Some(WindowPositionSizeReportMode::Inner),
        };
        let mut buf = Vec::new();
        cmd.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "\x1b[14;2t");
    }

    #[test]
    fn test_report_screen_size_pixels() {
        assert_eq!(ReportScreenSizePixels::BYTES, b"\x1b[15t");
    }

    #[test]
    fn test_report_cell_size_pixels() {
        assert_eq!(ReportCellSizePixels::BYTES, b"\x1b[16t");
    }

    #[test]
    fn test_report_size() {
        assert_eq!(ReportSize::BYTES, b"\x1b[18t");
    }

    #[test]
    fn test_report_screen_size() {
        assert_eq!(ReportScreenSize::BYTES, b"\x1b[19t");
    }

    #[test]
    fn test_window_state_report_not_iconified() {
        let report = WindowStateReport {
            state: WindowState::NotIconified,
        };
        let mut buf = Vec::new();
        report.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "\x1b[1t");
    }

    #[test]
    fn test_window_state_report_iconified() {
        let report = WindowStateReport {
            state: WindowState::Iconified,
        };
        let mut buf = Vec::new();
        report.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "\x1b[2t");
    }

    #[test]
    fn test_window_position_report() {
        let report = WindowPositionReport { x: 50, y: 100 };
        let mut buf = Vec::new();
        report.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "\x1b[3;50;100t");
    }

    #[test]
    fn test_window_size_pixels_report() {
        let report = WindowSizePixelsReport {
            height: 768,
            width: 1024,
        };
        let mut buf = Vec::new();
        report.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "\x1b[4;768;1024t");
    }

    #[test]
    fn test_screen_size_pixels_report() {
        let report = ScreenSizePixelsReport {
            height: 1080,
            width: 1920,
        };
        let mut buf = Vec::new();
        report.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "\x1b[5;1080;1920t");
    }

    #[test]
    fn test_cell_size_pixels_report() {
        let report = CellSizePixelsReport {
            height: 16,
            width: 8,
        };
        let mut buf = Vec::new();
        report.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "\x1b[6;16;8t");
    }

    #[test]
    fn test_size_report() {
        let report = SizeReport {
            rows: 30,
            cols: 120,
        };
        let mut buf = Vec::new();
        report.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "\x1b[8;30;120t");
    }

    #[test]
    fn test_screen_size_report() {
        let report = ScreenSizeReport {
            rows: 40,
            cols: 160,
        };
        let mut buf = Vec::new();
        report.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "\x1b[9;40;160t");
    }

    // ========================================================================
    // Title Mode Features Tests
    // ========================================================================

    #[test]
    fn test_set_title_mode_features_utf8() {
        let cmd = SetTitleModeFeatures(TitleModeFeatures::SET_UTF8);
        let mut buf = Vec::new();
        cmd.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "\x1b[>4t");
    }

    #[test]
    fn test_set_title_mode_features_hex() {
        let cmd = SetTitleModeFeatures(TitleModeFeatures::SET_HEX);
        let mut buf = Vec::new();
        cmd.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "\x1b[>1t");
    }

    #[test]
    fn test_set_title_mode_features_query_utf8() {
        let cmd = SetTitleModeFeatures(TitleModeFeatures::QUERY_UTF8);
        let mut buf = Vec::new();
        cmd.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "\x1b[>8t");
    }

    #[test]
    fn test_set_title_mode_features_combined() {
        let cmd = SetTitleModeFeatures(
            TitleModeFeatures::SET_UTF8 | TitleModeFeatures::QUERY_UTF8,
        );
        let mut buf = Vec::new();
        cmd.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "\x1b[>12t");
    }

    #[test]
    fn test_reset_title_mode_features_utf8() {
        let cmd = ResetTitleModeFeatures(TitleModeFeatures::SET_UTF8);
        let mut buf = Vec::new();
        cmd.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "\x1b[>4T");
    }

    #[test]
    fn test_reset_title_mode_features_hex() {
        let cmd = ResetTitleModeFeatures(TitleModeFeatures::SET_HEX);
        let mut buf = Vec::new();
        cmd.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "\x1b[>1T");
    }

    #[test]
    fn test_report_title_stack_position() {
        assert_eq!(ReportTitleStackPosition::BYTES, b"\x1b[#S");
    }

    #[test]
    fn test_title_stack_position_report_encoding() {
        let report = TitleStackPositionReport { position: 3 };
        let mut buf = Vec::new();
        report.encode_ansi_into(&mut buf).unwrap();
        // CSI parameters come before intermediate characters
        assert_eq!(String::from_utf8(buf).unwrap(), "\x1b[3#S");
    }

    #[test]
    fn test_title_stack_position_report_empty() {
        let report = TitleStackPositionReport { position: 0 };
        let mut buf = Vec::new();
        report.encode_ansi_into(&mut buf).unwrap();
        // CSI parameters come before intermediate characters
        assert_eq!(String::from_utf8(buf).unwrap(), "\x1b[0#S");
    }
}
