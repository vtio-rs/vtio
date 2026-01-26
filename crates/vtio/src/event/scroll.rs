//! Terminal scrolling control and scrolling region management.
//!
//! ## Scrolling in Terminal Emulators
//!
//! Scrolling is the movement of display content up or down within a
//! defined region of the screen, creating space for new content at the
//! opposite edge. Terminal scrolling operates on a "scrolling region"
//! (also called "scroll margins"), which defines the vertical boundaries
//! within which scrolling operations take place.
//!
//! ## Scrolling Region (Margins)
//!
//! The scrolling region is defined by top and bottom margins (vertical)
//! and optionally left and right margins (horizontal). When margins are
//! set, scrolling operations only affect content within these
//! boundaries.
//!
//! Key characteristics:
//! - The minimum scrolling region is two lines (top margin line number
//!   must be less than bottom margin line number)
//! - By default (or when reset), the scrolling region encompasses the
//!   entire screen
//! - Content outside the scrolling region remains unaffected by
//!   scrolling operations
//! - The cursor moves to the home position when margins change
//! - The home position is determined by the origin mode setting
//!   ([`RelativeCursorOriginMode`](crate::event::cursor::RelativeCursorOriginMode))
//!
//! ## Scroll Types
//!
//! Terminals support two scrolling methods:
//!
//! - **Jump scroll**: Lines appear immediately (as fast as the terminal
//!   can process them)
//! - **Smooth scroll**: Lines scroll gradually at a fixed rate (e.g., 6
//!   lines/second at 60Hz, 5 lines/second at 50Hz)
//!
//! ## Implicit Scrolling
//!
//! Scrolling can occur implicitly when:
//! - Cursor moves beyond the bottom margin (scrolls up)
//! - Cursor moves beyond the top margin (scrolls down)
//! - Line feed, form feed, or vertical tab is received at bottom margin
//! - Reverse index is performed at top margin
//!
//! ## Origin Mode Interaction
//!
//! When origin mode ([`RelativeCursorOriginMode`](crate::event::cursor::RelativeCursorOriginMode))
//! is set:
//! - Line numbers start at the top margin of the scrolling region
//! - The home position is at the top-left of the scrolling region
//! - The cursor cannot move outside the scrolling region
//!
//! When origin mode is reset:
//! - Line numbers are absolute (independent of scrolling region)
//! - The home position is at the upper-left corner of the screen
//! - The cursor can move outside the scrolling region using explicit
//!   positioning commands

crate::terminal_mode!(
    /// Smooth Scroll Mode (`DECSCLM`).
    ///
    /// # Sequence
    ///
    /// `CSI ? 4 h` (set) / `CSI ? 4 l` (reset)
    ///
    /// Control whether scrolling operations use smooth scrolling or
    /// jump scrolling.
    ///
    /// When set, smooth scroll is enabled. The terminal adds lines to
    /// the screen gradually at a fixed rate.
    ///
    /// When reset, jump scroll is enabled. The terminal adds lines to
    /// the screen as fast as possible.
    ///
    /// See <https://terminalguide.namepad.de/mode/p4/> for terminal
    /// support specifics.
    SmoothScrollMode, private = '?', params = ["4"]
);

crate::terminal_mode!(
    /// Left/Right Margin Mode (`DECLRMM`).
    ///
    /// # Sequence
    ///
    /// `CSI ? 69 h` (set) / `CSI ? 69 l` (reset)
    ///
    /// Control whether left and right margins are enabled.
    ///
    /// When set, left/right margin mode is enabled. The
    /// [`SetLeftAndRightMargins`] sequence can be used to define
    /// horizontal margins, and horizontal scrolling operations respect
    /// these margins.
    ///
    /// When reset, left/right margin mode is disabled. Horizontal
    /// margins are ignored, and [`SetLeftAndRightMargins`] has no
    /// effect. The `CSI Ps ; Ps s` sequence is interpreted as Save
    /// Cursor (SCOSC) instead.
    ///
    /// This mode affects:
    /// - Horizontal scrolling operations ([`ScrollLeft`], [`ScrollRight`])
    /// - Character insertion and deletion
    /// - Line wrapping behavior
    /// - Cursor movement restrictions (when origin mode is also set)
    ///
    /// See <https://vt100.net/docs/vt510-rm/DECLRMM.html> for the VT510
    /// reference manual entry.
    LeftRightMarginMode, private = '?', params = ["69"]
);

/// Set Top and Bottom Margins (`DECSTBM`).
///
/// *Sequence*: `CSI Ps ; Ps r`
///
/// Where the first `Ps` is the top margin and the second `Ps` is the
/// bottom margin. Default values reset to full screen.
///
/// Define the vertical scrolling region by specifying the top and
/// bottom margins. The scrolling region is the area between these
/// margins where scrolling operations take place.
///
/// After margins are set, the cursor moves to the home position. The
/// home position depends on the origin mode setting
/// ([`RelativeCursorOriginMode`](crate::event::cursor::RelativeCursorOriginMode)).
///
/// The minimum scrolling region is two lines, so `top` must be less
/// than `bottom`. Line numbers start at 1.
///
/// If either margin is set to 0, or if this sequence is sent with no
/// parameters, the scrolling region is reset to the full screen.
///
/// When the number of columns changes (e.g., switching between 80 and
/// 132 column mode), the scrolling region is automatically reset to the
/// full screen.
///
/// See <https://terminalguide.namepad.de/seq/csi_r/> and
/// <https://vt100.net/docs/vt102-ug/chapter5.html> for terminal support
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
#[vtansi(csi, finalbyte = 'r')]
pub struct SetTopAndBottomMargins {
    pub top: u16,
    pub bottom: u16,
}

/// Set Left and Right Margins (`DECSLRM`).
///
/// *Sequence*: `CSI Ps ; Ps s`
///
/// Where the first `Ps` is the left margin and the second `Ps` is the
/// right margin. Default values reset to full screen width.
///
/// Define the horizontal scrolling region by specifying the left and
/// right margins. This feature is part of the DEC private mode set and
/// is not as widely supported as vertical margins.
///
/// The left margin must be less than the right margin. Column numbers
/// start at 1.
///
/// If either margin is set to 0, or if this sequence is sent with no
/// parameters, the horizontal margins are reset to encompass the full
/// screen width.
///
/// This feature requires that left/right margin mode is enabled.
///
/// See <https://terminalguide.namepad.de/seq/csi_s_u/> for terminal
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
#[vtansi(csi, finalbyte = 's')]
pub struct SetLeftAndRightMargins {
    pub left: u16,
    pub right: u16,
}

/// Request top and bottom margins (`DECRQSS` - `DECSTBM`).
///
/// *Sequence*: `DCS $ q r ST`
///
/// Request the current vertical scrolling region (top and bottom
/// margins) from the terminal using DECRQSS (Request Selection or
/// Setting) to query the DECSTBM (Set Top and Bottom Margins) setting.
///
/// The terminal responds with a DCS sequence containing the current
/// margin settings.
///
/// The terminal responds with `DCS 1 $ r Pt ; Pb r ST` where `Pt` is
/// the top margin line number and `Pb` is the bottom margin line number.
///
/// See <https://terminalguide.namepad.de/seq/dcs-dollar-q-r/> for
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
#[vtansi(dcs, intermediate = "$", finalbyte = 'r')]
pub struct RequestTopBottomMargins;

/// Request left and right margins (`DECRQSS` - `DECSLRM`).
///
/// *Sequence*: `DCS $ q s ST`
///
/// Request the current horizontal scrolling region (left and right
/// margins) from the terminal using DECRQSS (Request Selection or
/// Setting) to query the DECSLRM (Set Left and Right Margins) setting.
///
/// The terminal responds with a DCS sequence containing the current
/// margin settings.
///
/// The terminal responds with `DCS 1 $ r Pl ; Pr s ST` where `Pl` is
/// the left margin column number and `Pr` is the right margin column number.
///
/// See <https://terminalguide.namepad.de/seq/dcs-dollar-q-s/> for
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
#[vtansi(dcs, intermediate = "$", finalbyte = 's')]
pub struct RequestLeftRightMargins;

/// Scroll Up (`SU`).
///
/// *Sequence*: `CSI Ps S`
///
/// Scroll the scrolling region up by the specified number of lines. New
/// blank lines with current attributes appear at the bottom of the
/// scrolling region. Lines scrolled off the top are lost.
///
/// The cursor position does not change.
///
/// If the parameter is 0 or not specified, scrolls up by 1 line.
///
/// Content outside the scrolling region is not affected.
///
/// See <https://terminalguide.namepad.de/seq/csi_s__u/> for terminal
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
#[vtansi(csi, finalbyte = 'S')]
pub struct ScrollUp(pub u16);

/// Scroll Down (`SD`).
///
/// *Sequence*: `CSI Ps T`
///
/// Scroll the scrolling region down by the specified number of lines.
/// New blank lines with current attributes appear at the top of the
/// scrolling region. Lines scrolled off the bottom are lost.
///
/// The cursor position does not change.
///
/// If the parameter is 0 or not specified, scrolls down by 1 line.
///
/// Content outside the scrolling region is not affected.
///
/// See <https://terminalguide.namepad.de/seq/csi_ct_1param/> for terminal
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
#[vtansi(csi, finalbyte = 'T')]
pub struct ScrollDown(pub u16);

/// Scroll Left (`SL`).
///
/// *Sequence*: `CSI Ps SP @`
///
/// Scroll the scrolling region left by the specified number of columns.
/// New blank columns with current attributes appear at the right edge
/// of the scrolling region. Columns scrolled off the left edge are
/// lost.
///
/// The cursor position does not change.
///
/// If the parameter is 0 or not specified, scrolls left by 1 column.
///
/// This feature requires horizontal scrolling support and is less
/// widely implemented than vertical scrolling.
///
/// See <https://terminalguide.namepad.de/seq/csi_sp_at/> for terminal
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
#[vtansi(csi, intermediate = " ", finalbyte = '@')]
pub struct ScrollLeft(pub u16);

/// Scroll Right (`SR`).
///
/// *Sequence*: `CSI Ps SP A`
///
/// Scroll the scrolling region right by the specified number of
/// columns. New blank columns with current attributes appear at the
/// left edge of the scrolling region. Columns scrolled off the right
/// edge are lost.
///
/// The cursor position does not change.
///
/// If the parameter is 0 or not specified, scrolls right by 1 column.
///
/// This feature requires horizontal scrolling support and is less
/// widely implemented than vertical scrolling.
///
/// See <https://terminalguide.namepad.de/seq/csi_sp_a/> for terminal
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
#[vtansi(csi, intermediate = " ", finalbyte = 'A')]
pub struct ScrollRight(pub u16);
