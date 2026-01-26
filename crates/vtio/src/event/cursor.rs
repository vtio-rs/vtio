//! Cursor movement and control commands.

use crate::terminal_mode;
use vtansi::{AnsiMuxEncode, bitflags, encode::EncodeError, parse::ParseError};

/// Define bitflags for DCS cursor information with 0x40 base offset.
///
/// This macro creates bitflags that encode/decode with a 0x40 offset and
/// convert to/from char for use in DCS data fields.
macro_rules! dcs_bitflags {
    (
        $(#[$attr:meta])*
        $vis:vis struct $name:ident: u8 {
            $($const_items:tt)*
        }
    ) => {
        bitflags! {
            $(#[$attr])*
            /// The base value has bit 7 set (0x40) in the encoded form.
            #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize), serde(transparent))]
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
            $vis struct $name: u8 {
                $($const_items)*
            }
            encode: |bits| (bits | 0x40) as char,
            decode: |bits| (bits as u8) & !0x40,
        }
    };
}

terminal_mode!(
    /// Cursor Origin Mode (`DECOM`).
    ///
    /// # Sequence
    ///
    /// `CSI ? 6 h` (set) / `CSI ? 6 l` (reset)
    ///
    /// If set, the origin of the coordinate system is relative to the
    /// current scroll region.
    ///
    /// The origin is used by cursor positioning commands such as
    /// [`SetCursorPosition`], [`CursorVerticalAbsolute`], [`CursorHorizontalAbsolute`], and
    /// cursor position reports.
    ///
    /// When this mode is set, certain sequences will force the cursor to be
    /// in the scrolling region, including carriage return, next line,
    /// cursor next/previous line operations.
    ///
    /// If set, the cursor is moved to the top left of the current scroll
    /// region.
    ///
    /// See <https://terminalguide.namepad.de/mode/p6/> for
    /// terminal support specifics.
    RelativeCursorOriginMode, private = '?', params = ["6"]
);

terminal_mode!(
    /// Cursor Blinking (`ATT610_BLINK`).
    ///
    /// # Sequence
    ///
    /// `CSI ? 12 h` (set) / `CSI ? 12 l` (reset)
    ///
    /// If set, the cursor is blinking.
    ///
    /// This mode interacts with the blinking part of the Select Cursor Style
    /// (`DECSCUSR`) setting. In xterm, this mode is synchronized with the
    /// blinking part of the cursor style. In urxvt, this mode is additive to
    /// the cursor style setting.
    ///
    /// See also [`SetCursorStyle`] for a more widely supported alternative.
    ///
    /// See <https://terminalguide.namepad.de/mode/p12/> for
    /// terminal support specifics.
    CursorBlinking, private = '?', params = ["12"]
);

terminal_mode!(
    /// Cursor Visibility Mode (`DECTCEM`).
    ///
    /// # Sequence
    ///
    /// `CSI ? 25 h` (set) / `CSI ? 25 l` (reset)
    ///
    /// Set visibility of the cursor.
    ///
    /// If set, the cursor is visible. If reset, the cursor is hidden.
    ///
    /// See <https://terminalguide.namepad.de/mode/p25/> for
    /// terminal support specifics.
    CursorVisibility, private = '?', params = ["25"]
);

/// Save cursor (`DECSC`).
///
/// *Sequence*: `ESC 7`
///
/// Save cursor position and other state.
///
/// The primary and alternate screen have distinct save state.
///
/// The following state is saved:
///   * the state of [`RelativeCursorOriginMode`]
///     (but not its saved state for restore mode);
///   * the current attributes;
///   * If newly printed characters are protected
///     (like start protected area or select character protection attribute);
///   * the current cursor position, relative to the
///     origin set via cursor origin;
///   * pending wrap state;
///   * GL and GR character sets;
///   * G0, G1, G2, G3 character sets.
///
/// One saved state is kept per screen (main / alternative).
/// If for the current screen state was already saved it is overwritten.
///
/// The state can be restored using [`RestoreCursor`].
///
/// See <https://terminalguide.namepad.de/seq/a_esc_a7/> for
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
#[vtansi(esc, finalbyte = '7')]
pub struct SaveCursor;

/// Restore cursor (`DECRC`).
///
/// *Sequence*: `ESC 8`
///
/// Restore cursor position and other state.
///
/// The primary and alternate screen have distinct save state.
///
/// The following state is restored:
///   * the state of [`RelativeCursorOriginMode`]
///     (but not its saved state for restore mode);
///   * the current attributes;
///   * If newly printed characters are protected
///     (like start protected area or select character protection attribute);
///   * the current cursor position, relative to the
///     origin set via cursor origin;
///   * pending wrap state;
///   * GL and GR character sets;
///   * G0, G1, G2, G3 character sets.
///
/// If no [`SaveCursor`] was done previously values are reset to their
/// hard reset values.
///
/// See <https://terminalguide.namepad.de/seq/a_esc_a8/> for
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
#[vtansi(esc, finalbyte = '8')]
pub struct RestoreCursor;

/// Backspace (`BS`).
///
/// *Sequence*: `0x08` (C0 control code)
///
/// Move the cursor one position to the left.
///
/// If the cursor is on the left-most column, the behavior is implementation
/// dependent (may stay in place or wrap to previous line).
///
/// This unsets the pending wrap state without wrapping.
///
/// See <https://terminalguide.namepad.de/seq/c_bs/> for
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
#[vtansi(c0, code = 0x08)]
pub struct Backspace;

/// Horizontal Tab (`TAB`).
///
/// *Sequence*: `0x09` (C0 control code)
///
/// Move the cursor to the next tab stop.
///
/// If there are no more tab stops, the cursor is moved to the right-most
/// column.
///
/// Tab stops can be set using [`HorizontalTabSet`].
///
/// See <https://terminalguide.namepad.de/seq/c_tab/> for
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
#[vtansi(c0, code = 0x09)]
pub struct HorizontalTab;

/// Line Feed (`LF`).
///
/// *Sequence*: `0x0A` (C0 control code)
///
/// Move the cursor to the next line.
///
/// The behavior depends on the Line Feed mode:
///   * If Line Feed mode is not set: move the cursor down one line
///     (like [`Index`])
///   * If Line Feed mode is set: move the cursor down one line and to the
///     left-most column (like [`NextLine`])
///
/// See <https://terminalguide.namepad.de/seq/c_lf/> for
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
#[vtansi(c0, code = 0x0A)]
pub struct LineFeed;

/// Vertical Tab (`VT`).
///
/// *Sequence*: `0x0B` (C0 control code)
///
/// Move the cursor down one line (same as [`LineFeed`]).
///
/// See <https://terminalguide.namepad.de/seq/c_vt/> for
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
#[vtansi(c0, code = 0x0B)]
pub struct VerticalTab;

/// Form Feed (`FF`).
///
/// *Sequence*: `0x0C` (C0 control code)
///
/// Move the cursor down one line (same as [`LineFeed`]).
///
/// See <https://terminalguide.namepad.de/seq/c_ff/> for
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
#[vtansi(c0, code = 0x0C)]
pub struct FormFeed;

/// Carriage Return (`CR`).
///
/// *Sequence*: `0x0D` (C0 control code)
///
/// Move the cursor to the left-most column.
///
/// This unsets the pending wrap state without wrapping.
///
/// If left and right margin mode is set and a left margin set and the cursor
/// is on or right of the left margin column it is moved to the left margin. If
/// the cursor is left of the left margin the cursor is moved to the left-most
/// column of the screen.
///
/// If a left margin is set and [`RelativeCursorOriginMode`] is set the cursor
/// will always move to the left margin column.
///
/// See <https://terminalguide.namepad.de/seq/c_cr/> for
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
#[vtansi(c0, code = 0x0D)]
pub struct CarriageReturn;

/// Set Cursor Position (`CUP`).
///
/// *Sequence*: `CSI Ps ; Ps H`
///
/// Where the first `Ps` is the row and the second `Ps` is the column.
/// Default values are 1 for both parameters.
///
/// Move cursor to the position indicated by `row` and `column`.
///
/// If `column` is 0, it is adjusted to 1. If `column` is greater than the
/// right-most column it is adjusted to the right-most column.
///
/// If `row` is 0, it is adjusted to 1. If `row` is greater than the
/// bottom-most row it is adjusted to the bottom-most row.
///
/// `column` = 1 is the left-most column. `row` = 1 is the top-most row.
///
/// This unsets the pending wrap state without wrapping.
///
/// If cursor origin mode is set the cursor row will be moved relative to the
/// top margin row and adjusted to be above or at bottom-most row in the
/// current scroll region.
///
/// If origin mode is set and left and right margin mode is set the cursor
/// will be moved relative to the left margin column and adjusted to be on or
/// left of the right margin column.
///
/// See <https://terminalguide.namepad.de/seq/csi_ch/> for
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
#[vtansi(csi, finalbyte = 'H')]
pub struct SetCursorPosition {
    pub row: u16,
    pub col: u16,
}

impl SetCursorPosition {
    #[must_use]
    pub fn new() -> Self {
        Self { row: 1, col: 1 }
    }
}

impl Default for SetCursorPosition {
    fn default() -> Self {
        Self::new()
    }
}

/// Back Index (`DECBI`).
///
/// *Sequence*: `ESC 6`
///
/// If the cursor is not on the left-most column of the scroll region this is
/// the same as [`CursorLeft`] with `amount = 1`.
///
/// If the cursor is on the left-most column of the scroll region and on a row
/// that is inside the scroll region, a new blank left-most column of the
/// scroll region is inserted. The previous content of the scroll region are
/// shifted to the right. The right-most column of the scroll region is
/// discarded. If the cell movement splits a multi cell character that
/// character cleared, by replacing it by spaces, keeping its attributes.
///
/// If the cursor is on the left-most column of the scroll region and on a row
/// that is outside the scroll region, nothing is changed.
///
/// The cleared space is colored according to the current SGR state.
///
/// Does not change the cursor position.
///
/// See <https://terminalguide.namepad.de/seq/a_esc_a6/> for
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
#[vtansi(esc, finalbyte = '6')]
pub struct BackIndex;

/// Forward Index (`DECFI`).
///
/// *Sequence*: `ESC 9`
///
/// If the cursor is not on the right-most column of the scroll region this is
/// the same as [`CursorRight`] with `amount = 1`.
///
/// If the cursor is on the right-most column of the scroll region and on a row
/// that is inside the scroll region, the whole left-most column of the scroll
/// region is deleted. The remaining characters are shifted to the left and
/// space from the right margin is filled with spaces. If the cell movement
/// splits a multi cell character that character is cleared, by replacing it by
/// spaces, keeping its attributes.
///
/// If the cursor is on the right-most column of the scroll region and on a row
/// that is outside the scroll region, nothing is changed.
///
/// The cleared space is colored according to the current SGR state.
///
/// Does not change the cursor position.
///
/// See <https://terminalguide.namepad.de/seq/a_esc_a9/> for
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
#[vtansi(esc, finalbyte = '9')]
pub struct ForwardIndex;

/// Index (`IND`).
///
/// *Sequence*: `ESC D`
///
/// Move the cursor to the next line in the scrolling region,
/// possibly scrolling.
///
/// If the cursor is outside of the scrolling region:
///   * move the cursor one line down if it is not on the
///     bottom-most line of the screen.
///
/// If the cursor is inside the scrolling region:
///   * if the cursor is on the bottom-most line of the scrolling region:
///     - invoke [`ScrollUp`](super::scroll::ScrollUp) with `amount=1`
///   * if the cursor is not on the bottom-most line of the scrolling region:
///     - move the cursor one line down
///
/// This unsets the pending wrap state without wrapping.
///
/// See <https://terminalguide.namepad.de/seq/a_esc_cd/> for
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
#[vtansi(esc, finalbyte = 'D')]
pub struct Index;

/// Next Line (`NEL`).
///
/// *Sequence*: `ESC E`
///
/// Send [`CarriageReturn`] and [`Index`].
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
#[vtansi(esc, finalbyte = 'E')]
pub struct NextLine;

/// Horizontal Tab Set (`HTS`).
///
/// *Sequence*: `ESC H`
///
/// Mark current column as tab stop column.
///
/// See <https://terminalguide.namepad.de/seq/a_esc_ch/> for
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
#[vtansi(esc, finalbyte = 'H')]
pub struct HorizontalTabSet;

/// Clear Tab Stop at Current Column (`TBC`).
///
/// *Sequence*: `CSI g` or `CSI 0 g`
///
/// Clears the tab stop at the current cursor column, if one is set.
///
/// Tab stops define positions where the cursor moves when a horizontal
/// tab character is processed. This command removes the tab stop at the
/// current column only; other tab stops remain unaffected.
///
/// If no tab stop is set at the current column, this command has no effect.
///
/// Use [`ClearAllTabStops`] to clear all tab stops at once.
///
/// See <https://vt100.net/docs/vt510-rm/TBC.html> for the VT510
/// specification.
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
#[vtansi(csi, finalbyte = 'g')]
pub struct ClearTabStop;

/// Clear All Tab Stops (`TBC`).
///
/// *Sequence*: `CSI 3 g`
///
/// Clears all tab stops that have been set.
///
/// After this command, there are no tab stops defined. The terminal
/// typically starts with default tab stops at every 8th column (1, 9,
/// 17, 25, ...). This command removes all of them.
///
/// New tab stops can be set using [`HorizontalTabSet`].
///
/// See <https://vt100.net/docs/vt510-rm/TBC.html> for the VT510
/// specification.
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
#[vtansi(csi, params = ["3"], finalbyte = 'g')]
pub struct ClearAllTabStops;

/// Reverse Index (`RI`).
///
/// *Sequence*: `ESC M`
///
/// Move the cursor to the previous line in the scrolling region,
/// possibly scrolling.
///
/// If the cursor is outside of the scrolling region:
///   * move the cursor one line up if it is not on the
///     top-most line of the screen.
///
/// If the cursor is inside the scrolling region:
///   * if the cursor is on the top-most line of the scrolling region:
///     - invoke [`ScrollDown`](super::scroll::ScrollDown) with `amount=1`
///   * if the cursor is not on the top-most line of the scrolling region:
///     - move the cursor one line up
///
/// This unsets the pending wrap state without wrapping.
///
/// See <https://terminalguide.namepad.de/seq/a_esc_cm/> for
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
#[vtansi(esc, finalbyte = 'M')]
pub struct ReverseIndex;

/// Cursor Up (`CUU`).
///
/// *Sequence*: `CSI Ps A`
///
/// Move cursor up by the specified `amount` of lines.
///
/// If `amount` is greater than the maximum move distance then it is
/// internally adjusted to the maximum. If `amount` is `0`, adjust it to `1`.
///
/// This unsets the pending wrap state without wrapping.
///
/// If the current scroll region is set and the cursor is on or below top-most
/// line of it then the cursor may move up only until it reaches the top-most
/// line of current scroll region.
///
/// If the current scroll region is not set or the cursor is above top-most
/// line of current scroll region it may move up until the top of the screen
/// (excluding scroll-back buffer).
///
/// See <https://terminalguide.namepad.de/seq/csi_ca/> for
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
#[vtansi(csi, finalbyte = 'A')]
pub struct CursorUp(pub u16);

/// Cursor Down (`CUD`).
///
/// *Sequence*: `CSI Ps B`
///
/// Move cursor down by the specified `amount` of lines.
///
/// If `amount` is greater than the maximum move distance then it is
/// internally adjusted to the maximum. This sequence will not scroll the
/// screen or scroll region. If `amount` is `0`, adjust it to `1`.
///
/// This unsets the pending wrap state without wrapping.
///
/// If the current scroll region is set and the cursor is on or above
/// bottom-most line of it then the cursor may move down only until it reaches
/// the bottom-most line of current scroll region.
///
/// If the current scroll region is not set or the cursor is below bottom-most
/// line of current scroll region it may move down until the bottom of the
/// screen.
///
/// See <https://terminalguide.namepad.de/seq/csi_cb/> for
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
#[vtansi(csi, finalbyte = 'B')]
pub struct CursorDown(pub u16);

/// Cursor Left (`CUB`).
///
/// *Sequence*: `CSI Ps D`
///
/// Move the cursor to the left `amount` cells.
///
/// If `amount` is 0, adjust it to 1.
///
/// This unsets the pending wrap state without wrapping.
///
/// If not both of reverse wrap mode and wraparound mode are set:
///   * Move the cursor `amount` cells left. If it would cross the left-most
///     column of the scrolling region, stop at the left-most column of the
///     scrolling region. If the cursor would move left of the left-most
///     column of the screen, move to the left most column of the screen.
///
/// Else:
///   * If the pending wrap state is set, reduce `amount` by one.
///   * If the cursor is left of the left-most column of the scrolling region:
///     - Move the cursor left `amount` of cells with the following rules:
///     - Each time the cursor is advanced past the left screen edge, continue
///       on the right-most column of the scrolling region on the line above.
///       If that would be before the top-most line of the screen resume on
///       the bottom most line of the screen (ignoring the top and bottom
///       margins of the scrolling region).
///   * If the cursor is on or right of the left-most column of the scrolling
///     region:
///     - Move the cursor left `amount` of cells with the following rules:
///     - Each time the cursor is advanced past the left-most column of the
///       scrolling region, continue on the right-most column of the scrolling
///       region on the line above. If that would be before the top-most line
///       of the screen resume on the bottom most line of the screen (ignoring
///       the top and bottom margins of the scrolling region).
///
/// See <https://terminalguide.namepad.de/seq/csi_cd/> for
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
#[vtansi(csi, finalbyte = 'D')]
pub struct CursorLeft(pub u16);

/// Cursor Right (`CUF`).
///
/// *Sequence*: `CSI Ps C`
///
/// Move the cursor right `amount` columns.
///
/// If `amount` is greater than the maximum move distance then it is
/// internally adjusted to the maximum. This sequence will not scroll the
/// screen or scroll region. If `amount` is 0, adjust it to 1.
///
/// This unsets the pending wrap state without wrapping.
///
/// If left and right margin mode is set and a right margin is set and the
/// cursor is on or left of the right-most column of it then the cursor may
/// move right only until it reaches the right-most column of current scroll
/// region.
///
/// If left and right margin mode is not set or a right margin is not set or
/// the cursor is right of right-most column of current scroll region it may
/// move right until the right-most column of the screen.
///
/// See <https://terminalguide.namepad.de/seq/csi_cc/> for
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
#[vtansi(csi, finalbyte = 'C')]
pub struct CursorRight(pub u16);

/// Cursor Next Line (`CNL`).
///
/// *Sequence*: `CSI Ps E`
///
/// Move `amount` lines down and to the beginning of the line.
///
/// If `amount` is 0, it is adjusted to 1.
///
/// This is a composition of cursor down with the given `amount` parameter
/// and carriage return.
///
/// See <https://terminalguide.namepad.de/seq/csi_ce/> for
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
#[vtansi(csi, finalbyte = 'E')]
pub struct CursorNextLine(pub u16);

/// Cursor Previous Line (`CPL`).
///
/// *Sequence*: `CSI Ps F`
///
/// Move `amount` lines up and to the beginning of the line.
///
/// If `amount` is 0, it is adjusted to 1.
///
/// This is a composition of cursor up with the given `amount` parameter
/// and carriage return.
///
/// See <https://terminalguide.namepad.de/seq/csi_cf/> for
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
#[vtansi(csi, finalbyte = 'F')]
pub struct CursorPreviousLine(pub u16);

/// Cursor Horizontal Absolute (`CHA`).
///
/// *Sequence*: `CSI Ps G`
///
/// Move the cursor to column `col` on the current line.
///
/// If `col` is 0, it is adjusted to 1. If `col` is greater than the
/// right-most column it is adjusted to the right-most column.
///
/// `col` = 1 is the left-most column.
///
/// This unsets the pending wrap state without wrapping.
///
/// See <https://terminalguide.namepad.de/seq/csi_cg/> for
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
#[vtansi(csi, finalbyte = 'G')]
pub struct CursorHorizontalAbsolute(pub u16);

/// Character Position Absolute (`HPA`).
///
/// *Sequence*: `CSI Ps `` ` (backtick)
///
/// Move the cursor to column `col` on the current line.
///
/// This is functionally equivalent to [`CursorHorizontalAbsolute`] (CHA),
/// but uses a different escape sequence. Both commands move the cursor
/// to an absolute column position on the current line.
///
/// If `col` is 0, it is adjusted to 1. If `col` is greater than the
/// right-most column, it is adjusted to the right-most column.
///
/// `col` = 1 is the left-most column.
///
/// This unsets the pending wrap state without wrapping.
///
/// See <https://vt100.net/docs/vt510-rm/HPA.html> for the VT510
/// specification.
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
#[vtansi(csi, finalbyte = '`')]
pub struct CharacterPositionAbsolute(pub u16);

/// Horizontal and Vertical Position (`HVP`).
///
/// *Sequence*: `CSI Ps ; Ps f`
///
/// Move the cursor to the specified row and column.
///
/// This is functionally equivalent to [`SetCursorPosition`] (CUP),
/// but uses a different escape sequence (`f` instead of `H`). Both
/// commands position the cursor at an absolute screen location.
///
/// If `row` or `col` is 0, it is adjusted to 1. If `row` or `col` is
/// greater than the screen dimensions, it is adjusted to the maximum.
///
/// `row` = 1, `col` = 1 is the top-left corner of the screen.
///
/// This unsets the pending wrap state without wrapping.
///
/// See <https://vt100.net/docs/vt510-rm/HVP.html> for the VT510
/// specification.
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
#[vtansi(csi, finalbyte = 'f')]
pub struct HorizontalVerticalPosition {
    pub row: u16,
    pub col: u16,
}

impl HorizontalVerticalPosition {
    #[must_use]
    pub fn new() -> Self {
        Self { row: 1, col: 1 }
    }
}

impl Default for HorizontalVerticalPosition {
    fn default() -> Self {
        Self::new()
    }
}

/// Cursor Horizontal Forward Tabulation (`CHT`).
///
/// *Sequence*: `CSI Ps I`
///
/// Invoke horizontal tab `amount` times.
///
/// Move cursor to the `amount`-th next tab stop.
///
/// Repeat the following procedure `amount` times:
///
/// Move the cursor right until it reaches a column marked as tab stop
/// (that is not the column the cursor started on) or the right-most
/// column of the screen.
///
/// If cursor origin is set and after this move the cursor is right of the
/// right-most column of the scrolling region, move the cursor to the
/// right-most column of the scrolling region.
///
/// See <https://terminalguide.namepad.de/seq/csi_ci/> for
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
#[vtansi(csi, finalbyte = 'I')]
pub struct CursorHorizontalForwardTab {
    pub amount: u16,
}

/// Cursor Horizontal Backward Tabulation (`CBT`).
///
/// *Sequence*: `CSI Ps Z`
///
/// Move cursor to the `amount`-th previous tab stop.
///
/// Repeat the following procedure `amount` times:
///
/// Move the cursor left until it reaches a column marked as tab stop
/// (that is not the column the cursor started on) or the left-most
/// column of the screen.
///
/// If cursor origin is set and after this move the cursor is left of the
/// left-most column of the scrolling region, move the cursor to the
/// left-most column of the scrolling region.
///
/// See <https://terminalguide.namepad.de/seq/csi_cz/> for
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
#[vtansi(csi, finalbyte = 'Z')]
pub struct CursorHorizontalBackwardTab(pub u16);

/// Cursor Horizontal Position Relative (`HPR`).
///
/// *Sequence*: `CSI Ps a`
///
/// Move cursor right by the specified `amount` of columns.
///
/// If `amount` is greater than the maximum move distance then it is
/// internally adjusted to the maximum. This sequence will not scroll the
/// screen or scroll region. If `amount` is 0, adjust it to 1.
///
/// This unsets the pending wrap state without wrapping.
///
/// If left and right margin mode is set and a right margin is set and the
/// cursor is on or left of the right-most column of it then the cursor may
/// move right only until it reaches the right-most column of current scroll
/// region.
///
/// If left and right margin mode is not set or a right margin is not set or
/// the cursor is right of right-most column of current scroll region it may
/// move right until the right-most column of the screen.
///
/// See <https://terminalguide.namepad.de/seq/csi_ca/> for
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
#[vtansi(csi, finalbyte = 'a')]
pub struct CursorHorizontalRelative(pub u16);

/// Cursor Vertical Position Absolute (`VPA`).
///
/// *Sequence*: `CSI Ps d`
///
/// Move the cursor to row `row` on the current column.
///
/// If `row` is 0, it is adjusted to 1. If `row` is greater than the
/// bottom-most row it is adjusted to the bottom-most row.
///
/// `row` = 1 is the top-most row.
///
/// This unsets the pending wrap state without wrapping.
///
/// If cursor origin mode is set the cursor row will be moved relative to the
/// top margin row and adjusted to be above or at bottom-most row in the
/// current scroll region.
///
/// See <https://terminalguide.namepad.de/seq/csi_cd/> for
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
#[vtansi(csi, finalbyte = 'd')]
pub struct CursorVerticalAbsolute(pub u16);

/// Vertical Position Relative (`VPR`).
///
/// *Sequence*: `CSI Ps e`
///
/// Move cursor down by the specified `amount` of lines.
///
/// If `amount` is greater than the maximum move distance then it is
/// internally adjusted to the maximum. This sequence will not scroll the
/// screen or scroll region. If `amount` is 0, adjust it to 1.
///
/// This unsets the pending wrap state without wrapping.
///
/// If the current scroll region is set and the cursor is on or above
/// bottom-most line of it then the cursor may move down only until it reaches
/// the bottom-most line of current scroll region.
///
/// If the current scroll region is not set or the cursor is below bottom-most
/// line of current scroll region it may move down until the bottom of the
/// screen.
///
/// See <https://terminalguide.namepad.de/seq/csi_ce/> for
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
#[vtansi(csi, finalbyte = 'e')]
pub struct CursorVerticalRelative(pub u16);

/// Cursor style variants for `DECSCUSR`.
///
/// These control the visual appearance of the cursor.
///
/// See <https://terminalguide.namepad.de/seq/csi_cq/> for
/// terminal support specifics.
#[derive(
    Debug,
    Default,
    PartialOrd,
    PartialEq,
    Eq,
    Clone,
    Copy,
    Hash,
    num_enum::TryFromPrimitive,
    num_enum::IntoPrimitive,
    vtansi::derive::FromAnsi,
    vtansi::derive::ToAnsi,
)]
#[repr(u8)]
pub enum CursorStyle {
    /// Default cursor style (usually blinking block).
    #[default]
    Default = 0,
    /// Blinking block cursor.
    BlinkingBlock = 1,
    /// Steady (non-blinking) block cursor.
    SteadyBlock = 2,
    /// Blinking underline cursor.
    BlinkingUnderline = 3,
    /// Steady underline cursor.
    SteadyUnderline = 4,
    /// Blinking bar (vertical line) cursor.
    BlinkingBar = 5,
    /// Steady bar cursor.
    SteadyBar = 6,
}

/// Select Cursor Style (`DECSCUSR`).
///
/// *Sequence*: `CSI Ps SP q`
///
/// Set the cursor style (shape and blinking).
///
/// The cursor style is set using values 0-6:
///   * 0 - Default cursor style (usually blinking block)
///   * 1 - Blinking block
///   * 2 - Steady block
///   * 3 - Blinking underline
///   * 4 - Steady underline
///   * 5 - Blinking bar
///   * 6 - Steady bar
///
/// See <https://terminalguide.namepad.de/seq/csi_sq_t_space/> for
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
#[vtansi(csi, intermediate = " ", finalbyte = 'q')]
pub struct SetCursorStyle {
    pub style: CursorStyle,
}

/// Request Cursor Style (`DECRQSS`).
///
/// *Sequence*: `DCS $ q SP q ST`
///
/// Request the current cursor style.
///
/// The terminal will respond with a DCS sequence containing the current
/// cursor style setting.
///
/// See <https://terminalguide.namepad.de/seq/dcs-dollar-q-space-q/> for
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
#[vtansi(dcs, intermediate = "$", finalbyte = 'q', data = " q")]
pub struct RequestCursorStyle;

/// Cursor Style Report (`DECRQSS` response).
///
/// Response from the terminal to [`RequestCursorStyle`].
///
/// Contains the current cursor style setting as reported via DECRQSS.
///
/// The response format is:
/// `DCS 1 $ r Ps SP q ST`
///
/// Where `Ps` is the cursor style value (0-6).
///
/// A response with parameter `0` instead of `1` indicates the request
/// was not recognized.
///
/// See <https://terminalguide.namepad.de/seq/dcs-dollar-q-space-q/> for
/// terminal support specifics.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, vtansi::derive::AnsiInput,
)]
#[vtansi(dcs, locate_all = "data", params = ["1"], intermediate = "$", finalbyte = 'r')]
pub struct CursorStyleReport {
    /// The cursor style value.
    pub style: CursorStyleReportData,
}

/// Wrapper for parsing cursor style from DECRQSS response data.
///
/// The response data format is `Ps SP q` where Ps is the cursor style value.
/// This wrapper parses that format and extracts the [`CursorStyle`] value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct CursorStyleReportData(pub CursorStyle);

impl vtansi::parse::TryFromAnsi<'_> for CursorStyleReportData {
    fn try_from_ansi(bytes: &[u8]) -> Result<Self, ParseError> {
        // Expected format: "Ps q" where Ps is a number (0-6)
        // The space is 0x20 and q is 0x71
        // So we need at least 3 bytes: digit, space, 'q'
        if bytes.len() < 3 {
            return Err(ParseError::WrongLen {
                expected: 3,
                got: bytes.len(),
            });
        }

        // Check that it ends with " q" (space + 'q')
        if bytes[bytes.len() - 2] != b' ' || bytes[bytes.len() - 1] != b'q' {
            return Err(ParseError::InvalidValue(
                "cursor style response must end with ' q'".to_string(),
            ));
        }

        // Parse the numeric value (everything before " q")
        let num_bytes = &bytes[..bytes.len() - 2];
        let num: u8 = vtansi::parse::TryFromAnsi::try_from_ansi(num_bytes)?;

        // Convert to CursorStyle
        let style = CursorStyle::try_from(num).map_err(|_| {
            ParseError::InvalidValue(format!(
                "invalid cursor style value: {num}"
            ))
        })?;

        Ok(CursorStyleReportData(style))
    }
}

impl vtansi::encode::AnsiEncode for CursorStyleReportData {
    fn encode_ansi_into<W: std::io::Write + ?Sized>(
        &self,
        writer: &mut W,
    ) -> Result<usize, EncodeError> {
        let style_byte: u8 = self.0.into();
        let mut len = 0;
        len += vtansi::encode::write_int(writer, style_byte)?;
        len += vtansi::encode::write_bytes_into(writer, b" q")?;
        Ok(len)
    }
}

impl From<CursorStyle> for CursorStyleReportData {
    fn from(style: CursorStyle) -> Self {
        Self(style)
    }
}

impl From<CursorStyleReportData> for CursorStyle {
    fn from(data: CursorStyleReportData) -> Self {
        data.0
    }
}

bitflags! {
    /// Flags for Linux cursor style.
    ///
    /// These flags control the cursor appearance and behavior in the Linux
    /// virtual console.
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize), serde(transparent))]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct LinuxCursorStyleFlags: u8 {
        /// Enable foreground and background change.
        ///
        /// When enabled the cursor changes the shown attributes of the
        /// cell it is on. Some drivers force size = 1 (none) internally
        /// if this is set.
        const ENABLE_FG_BG_CHANGE = 16;

        /// Ensure original background and cursor background differ.
        ///
        /// If the original and cursor background would be identical
        /// invert all background color channels (but not brightness).
        const ENSURE_BG_DIFFERS = 32;

        /// Ensure cursor foreground and background differ.
        ///
        /// If the cursor background and foreground would be identical
        /// invert all foreground color channels (but not brightness).
        const ENSURE_FG_BG_DIFFER = 64;
    }
}

impl AnsiMuxEncode for LinuxCursorStyleFlags {
    type BaseType = u8;

    fn mux_encode(
        &self,
        base: Option<&Self::BaseType>,
    ) -> Result<Self::BaseType, vtansi::EncodeError> {
        Ok(self.bits() | base.map_or(0, |b| *b))
    }
}

/// Linux Cursor Style shape values.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Default,
    num_enum::TryFromPrimitive,
    num_enum::IntoPrimitive,
    vtansi::derive::FromAnsi,
    vtansi::derive::ToAnsi,
)]
#[repr(u8)]
pub enum LinuxCursorSize {
    /// Default (depending on driver: off, underline or block).
    #[default]
    Default = 0,
    /// No cursor.
    None = 1,
    /// Underline cursor.
    Underline = 2,
    /// Lower third cursor.
    LowerThird = 3,
    /// Lower half cursor.
    LowerHalf = 4,
    /// Two thirds cursor.
    TwoThirds = 5,
    /// Block cursor.
    Block = 6,
}

impl AnsiMuxEncode for LinuxCursorSize {
    type BaseType = u8;

    fn mux_encode(
        &self,
        base: Option<&Self::BaseType>,
    ) -> Result<Self::BaseType, vtansi::EncodeError> {
        Ok((*self as u8) | base.map_or(0, |b| *b))
    }
}

/// Linux Cursor Style.
///
/// *Sequence*: `CSI ? Ps ; Ps ; Ps c`
///
/// Select Linux cursor style with fine-grained control over appearance.
///
/// This sequence allows setting the cursor shape, flags for attribute
/// changes, and XOR/OR masks for foreground and background color
/// manipulation.
///
/// The `shape` parameter combines the size (0-6) with optional
/// flags (16, 32, 64).
///
/// The `xor` and `or` parameters define changes to foreground and
/// background of the cell where the cursor is shown when the
/// `ENABLE_FG_BG_CHANGE` flag is set. Each bit controls one color channel:
///
/// | bit value |          meaning              |
/// |-----------|-------------------------------|
/// |         1 | foreground blue channel       |
/// |         2 | foreground green channel      |
/// |         4 | foreground red channel        |
/// |         8 | foreground brightness channel |
/// |        16 | background blue channel       |
/// |        32 | background green channel      |
/// |        64 | background red channel        |
/// |       128 | background brightness         |
///
/// The effective change for each bit depends on its value in both
/// parameters:
///
/// | or bit | xor bit |   change  |
/// |--------|---------|-----------|
/// |    0   |    0    | no change |
/// |    1   |    0    | enable    |
/// |    0   |    1    | toggle    |
/// |    1   |    1    | disable   |
///
/// See <https://terminalguide.namepad.de/seq/csi_sc__p/> for terminal
/// support specifics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, vtansi::derive::AnsiOutput)]
#[vtansi(csi, private = '?', finalbyte = 'c')]
pub struct LinuxCursorStyle {
    /// Cursor size.
    pub size: LinuxCursorSize,
    /// Cursor style flags.
    #[vtansi(muxwith = "size")]
    pub flags: LinuxCursorStyleFlags,
    /// XOR mask for color channel manipulation.
    pub xor: u8,
    /// OR mask for color channel manipulation.
    pub or: u8,
}

/// Request Cursor Position Report (`CPR`).
///
/// *Sequence*: `CSI 6 n`
///
/// Request the current cursor position.
///
/// The terminal replies with `CSI Ps ; Ps R` where the first `Ps` is
/// the row and the second `Ps` is the column.
///
/// If [`RelativeCursorOriginMode`] is set, the cursor position is reported
/// relative to the top left corner of the scroll area. Otherwise, it is
/// reported relative to the top left corner of the screen.
///
/// The response uses [`CursorPositionReport`].
///
/// See <https://terminalguide.namepad.de/seq/csi_sn-6/> for
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
#[vtansi(csi, params = ["6"], finalbyte = 'n')]
pub struct RequestCursorPosition;

/// Cursor Position Report (`CPR`).
///
/// *Sequence*: `CSI Ps ; Ps R`
///
/// Response from the terminal to [`RequestCursorPosition`].
///
/// Contains the current cursor position as `row` and `col`.
///
/// The position may be relative to the scroll area if
/// [`RelativeCursorOriginMode`] is set, or relative to the screen otherwise.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, vtansi::derive::AnsiInput,
)]
#[vtansi(csi, finalbyte = 'R')]
pub struct CursorPositionReport {
    pub row: u16,
    pub col: u16,
}

/// Request Cursor Information Report (`DECCIR`).
///
/// *Sequence*: `CSI 1 $ w`
///
/// Request detailed cursor information including position, attributes,
/// protection status, flags, and character set information.
///
/// The terminal replies with a DCS sequence containing:
/// - Cursor position (row and column)
/// - Current text attributes (bold, underline, blink, inverse)
/// - Character protection status
/// - Various cursor flags (origin mode, single shift, pending wrap)
/// - Character set information (GL, GR, and G0-G3 sets)
///
/// See <https://terminalguide.namepad.de/seq/csi_sw_t_dollar-1/> for
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
#[vtansi(csi, params = ["1"], intermediate = "$", finalbyte = 'w')]
pub struct RequestCursorInformationReport;

dcs_bitflags! {
    /// Cursor attribute flags for cursor information report.
    ///
    /// These flags encode the currently active text attributes.
    pub struct CursorAttributes: u8 {
        /// Bold text attribute.
        const BOLD = 0x01;
        /// Underline text attribute.
        const UNDERLINE = 0x02;
        /// Blink text attribute.
        const BLINK = 0x04;
        /// Inverse (reverse video) text attribute.
        const INVERSE = 0x08;
    }
}

dcs_bitflags! {
    /// Cursor state flags for cursor information report.
    ///
    /// These flags encode various cursor and terminal state information.
    pub struct CursorStateFlags: u8 {
        /// Cursor origin mode is set.
        const ORIGIN_MODE = 1;
        /// Single shift for G2 is active.
        const SINGLE_SHIFT_G2 = 2;
        /// Single shift for G3 is active.
        const SINGLE_SHIFT_G3 = 4;
        /// Pending wrap is set.
        const PENDING_WRAP = 8;
    }
}

dcs_bitflags! {
    /// Character set sizes for cursor information report (Scss).
    ///
    /// Indicates whether each G0-G3 character set has 94 or 96 characters.
    /// Bits 1-4 indicate which sets have 96 characters (0 = 94 characters,
    /// 1 = 96 characters).
    pub struct CharacterSetSizes: u8 {
        /// G0 character set has 96 characters (otherwise 94).
        const G0_96 = 1;
        /// G1 character set has 96 characters (otherwise 94).
        const G1_96 = 2;
        /// G2 character set has 96 characters (otherwise 94).
        const G2_96 = 4;
        /// G3 character set has 96 characters (otherwise 94).
        const G3_96 = 8;
    }
}

/// Cursor Information Report (`DECCIR` response).
///
/// *Sequence*: `DCS 1 $ u Pr ; Pc ; Pp ; Srend ; Satt ; Sflag ; Pgl ; Pgr ; Scss ; Sdesig ST`
///
/// Response from the terminal to [`RequestCursorInformationReport`].
///
/// Contains detailed information about the cursor state including
/// position, attributes, protection, flags, and character set
/// configuration.
///
/// See <https://vt100.net/docs/vt510-rm/DECCIR> for the VT510 specification.
/// See <https://terminalguide.namepad.de/seq/csi_sw_t_dollar-1/> for
/// terminal support specifics.
#[derive(Debug, Clone, PartialEq, Eq, vtansi::derive::AnsiInput)]
#[vtansi(dcs, locate_all = "data", params = ["1"], intermediate = "$", finalbyte = 'u')]
pub struct CursorInformationReport<'a> {
    /// Cursor row position (Pr).
    pub row: u16,
    /// Cursor column position (Pc).
    pub col: u16,
    /// Current page number (Pp).
    pub page: u8,
    /// Current text attributes (Srend).
    ///
    /// Visual attributes such as bold, underline, blink, and reverse video.
    pub attributes: CursorAttributes,
    /// Character protection attribute (Satt).
    ///
    /// Indicates selective erase protection status.
    pub protection_char: char,
    /// Cursor state flags (Sflag).
    ///
    /// Includes origin mode, single shift settings, and autowrap pending.
    pub flags: CursorStateFlags,
    /// Character set invoked into GL (Pgl): 0-3 for G0-G3.
    pub gl: u8,
    /// Character set invoked into GR (Pgr): 0-3 for G0-G3.
    pub gr: u8,
    /// Character set sizes (Scss).
    ///
    /// Indicates whether each G0-G3 set has 94 or 96 characters.
    pub charset_sizes: CharacterSetSizes,
    /// Character set designations (Sdesig).
    ///
    /// String of intermediate and final characters indicating the character
    /// sets designated as G0 through G3.
    pub gsets: &'a str,
}

/// Owned version of [`CursorInformationReport`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CursorInformationReportOwned {
    pub row: u16,
    pub col: u16,
    pub page: u8,
    pub attributes: CursorAttributes,
    pub protection_char: char,
    pub flags: CursorStateFlags,
    pub gl: u8,
    pub gr: u8,
    pub charset_sizes: CharacterSetSizes,
    pub gsets: String,
}

impl<'a> From<CursorInformationReport<'a>> for CursorInformationReportOwned {
    fn from(report: CursorInformationReport<'a>) -> Self {
        Self {
            row: report.row,
            col: report.col,
            page: report.page,
            attributes: report.attributes,
            protection_char: report.protection_char,
            flags: report.flags,
            gl: report.gl,
            gr: report.gr,
            charset_sizes: report.charset_sizes,
            gsets: report.gsets.to_string(),
        }
    }
}

impl CursorInformationReportOwned {
    /// Borrow this owned report as a borrowed version.
    #[must_use]
    pub fn borrow(&self) -> CursorInformationReport<'_> {
        CursorInformationReport {
            row: self.row,
            col: self.col,
            page: self.page,
            attributes: self.attributes,
            protection_char: self.protection_char,
            flags: self.flags,
            gl: self.gl,
            gr: self.gr,
            charset_sizes: self.charset_sizes,
            gsets: &self.gsets,
        }
    }
}

impl<'a> CursorInformationReport<'a> {
    /// Convert to an owned version.
    #[must_use]
    pub fn to_owned(&self) -> CursorInformationReportOwned {
        CursorInformationReportOwned {
            row: self.row,
            col: self.col,
            page: self.page,
            attributes: self.attributes,
            protection_char: self.protection_char,
            flags: self.flags,
            gl: self.gl,
            gr: self.gr,
            charset_sizes: self.charset_sizes,
            gsets: self.gsets.to_string(),
        }
    }

    /// Create a new cursor information report with the specified
    /// parameters.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn from_parts(
        row: u16,
        col: u16,
        page: u8,
        attributes: CursorAttributes,
        protected: bool,
        flags: CursorStateFlags,
        gl: u8,
        gr: u8,
        charset_sizes: CharacterSetSizes,
        gsets: &'a str,
    ) -> Self {
        Self {
            row,
            col,
            page,
            attributes,
            protection_char: if protected { 'A' } else { '@' },
            flags,
            gl,
            gr,
            charset_sizes,
            gsets,
        }
    }

    /// Check if character protection is enabled.
    #[must_use]
    pub const fn protected(&self) -> bool {
        matches!(self.protection_char, 'A')
    }
}

/// Request Tab Stop Report (`DECTABSR`).
///
/// *Sequence*: `CSI 2 $ w`
///
/// Request a report of the currently set tab stops.
///
/// The terminal replies with a DCS sequence containing the column
/// numbers of all set tab stops, separated by forward slashes (/).
///
/// All explicitly set tab stops and default tab stops that fit within
/// the current terminal width are reported.
///
/// See <https://terminalguide.namepad.de/seq/csi_sw_t_dollar-2/> for
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
#[vtansi(csi, params = ["2"], intermediate = "$", finalbyte = 'w')]
pub struct RequestTabStopReport;

/// Tab Stop Report (`DECTABSR` response).
///
/// *Sequence*: `DCS 2 $ u Pc / Pc / ... ST`
///
/// Response from the terminal to [`RequestTabStopReport`].
///
/// Contains the column numbers of all currently set tab stops,
/// where `Pc` values are tab stop column positions separated by `/`.
///
/// Example: `DCS 2 $ u 9/17/25/33 ST`
///
/// See <https://vt100.net/docs/vt510-rm/DECTABSR> for the VT510 specification.
/// See <https://terminalguide.namepad.de/seq/csi_sw_t_dollar-2/> for
/// terminal support specifics.
#[derive(Debug, Clone, PartialEq, Eq, vtansi::derive::AnsiInput)]
#[vtansi(dcs, locate_all = "data", params = ["2"], intermediate = "$", finalbyte = 'u')]
pub struct TabStopReport {
    /// Tab stop column positions, encoded as slash-separated string.
    pub tab_stops: TabStops,
}

/// Tab stops wrapper for encoding.
///
/// Encodes a vector of tab stop positions as a slash-separated string.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Default,
    vtansi::derive::FromAnsi,
    vtansi::derive::ToAnsi,
)]
#[vtansi(transparent, delimiter = b'/')]
pub struct TabStops(pub Vec<u16>);

impl From<Vec<u16>> for TabStops {
    fn from(stops: Vec<u16>) -> Self {
        Self(stops)
    }
}

impl From<&[u16]> for TabStops {
    fn from(stops: &[u16]) -> Self {
        Self(stops.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vtansi::encode::AnsiEncode;

    #[test]
    fn test_cursor_information_report_encoding() {
        // Test encoding of DECCIR (Cursor Information Report)
        // Format: DCS 1 $ u Pr; Pc; Pp; Srend; Satt; Sflag; Pgl; Pgr; Scss; Sdesig ST
        //
        // This test creates a report matching the VT510 documentation example:
        // - Cursor at row 10, column 20, page 1
        // - No visual attributes set (Srend = '@' = 0x40)
        // - No protection attributes (Satt = '@' = 0x40)
        // - No flags set: DECOM reset, no SS2/SS3, no autowrap pending (Sflag = '@' = 0x40)
        // - G0 mapped into GL (Pgl = 0)
        // - G2 mapped into GR (Pgr = 2)
        // - All character sets have 94 characters (Scss = '@' = 0x40)
        // - Character set designations: ASCII in G0/G1, DEC Supplemental in G2/G3
        //
        // Expected output: DCS 1 $ u 10;20;1;@;@;@;0;2;@;BB%5%5 ST
        let report = CursorInformationReport::from_parts(
            10,                         // Pr (row)
            20,                         // Pc (column)
            1,                          // Pp (page)
            CursorAttributes::empty(),  // Srend (no visual attributes)
            false,                      // Satt (not protected = '@')
            CursorStateFlags::empty(),  // Sflag (no flags)
            0,                          // Pgl (G0 in GL)
            2,                          // Pgr (G2 in GR)
            CharacterSetSizes::empty(), // Scss (all 94-char sets)
            "BB%5%5", // Sdesig (ASCII in G0/G1, DEC Supp in G2/G3)
        );

        let mut buf = Vec::new();
        let len = report.encode_ansi_into(&mut buf).unwrap();

        // Expected format: ESC P 1 $ u Pr; Pc; Pp; Srend; Satt; Sflag; Pgl; Pgr; Scss; Sdesig ESC \
        // ESC = 0x1B, P = 0x50, 1 = 0x31 (param), $ = 0x24, u = 0x75, \ = 0x5C
        assert_eq!(buf[0], 0x1B, "Should start with ESC");
        assert_eq!(buf[1], 0x50, "Should have P (DCS)");
        assert_eq!(buf[2], b'1', "Should have param '1'");
        assert_eq!(buf[3], 0x24, "Should have $ (intermediate)");
        assert_eq!(buf[4], 0x75, "Should have u (final byte)");

        // Check that the sequence contains the expected data
        let output = String::from_utf8_lossy(&buf);
        assert!(output.contains("10;"), "Should contain row (10)");
        assert!(output.contains(";20;"), "Should contain col (20)");
        assert!(output.contains(";1;"), "Should contain page (1)");
        assert!(
            output.contains(";@;"),
            "Should contain @ for attributes/protection/flags"
        );
        assert!(output.contains("0;2"), "Should contain gl (0) and gr (2)");
        assert!(output.contains("BB%5%5"), "Should contain gsets (BB%5%5)");

        // Check that it ends with ST (ESC \)
        assert_eq!(buf[len - 2], 0x1B, "Should end with ESC");
        assert_eq!(buf[len - 1], 0x5C, "Should end with backslash (ST)");

        // Verify the full expected format
        // DCS 1 $ u 10;20;1;@;@;@;0;2;@;BB%5%5 ST
        let expected_data = "10;20;1;@;@;@;0;2;@;BB%5%5";
        assert!(
            output.contains(expected_data),
            "Should contain expected data format, got: {output:?}",
        );

        // Verify length is reasonable
        assert!(len > 20, "Encoded length should be substantial");
    }

    #[test]
    fn test_tab_stop_report_encoding() {
        // Test encoding of DECTABSR (Tab Stop Report)
        // Format: DCS 2 $ u <data> ST where data is tab stops separated by /
        //
        // This test creates a report matching the VT510 documentation example:
        // - Tab stops at columns 9, 17, 25, 33, 41, 49, 57, 65, 73
        //
        // Expected output: DCS 2 $ u 9/17/25/33/41/49/57/65/73 ST
        let report = TabStopReport {
            tab_stops: vec![9, 17, 25, 33, 41, 49, 57, 65, 73].into(),
        };

        let mut buf = Vec::new();
        let len = report.encode_ansi_into(&mut buf).unwrap();

        // Expected format: ESC P 2 $ u <data> ESC \
        // ESC = 0x1B, P = 0x50, 2 = 0x32 (param), $ = 0x24, u = 0x75, \ = 0x5C
        assert_eq!(buf[0], 0x1B, "Should start with ESC");
        assert_eq!(buf[1], 0x50, "Should have P (DCS)");
        assert_eq!(buf[2], b'2', "Should have param '2'");
        assert_eq!(buf[3], 0x24, "Should have $ (intermediate)");
        assert_eq!(buf[4], 0x75, "Should have u (final byte)");

        // Check that the sequence contains the expected data
        let output = String::from_utf8_lossy(&buf);
        assert!(
            output.contains("9/17/25/33"),
            "Should contain tab stops separated by /"
        );
        assert!(
            output.contains("41/49/57/65/73"),
            "Should contain remaining tab stops"
        );

        // Check that it ends with ST (ESC \)
        assert_eq!(buf[len - 2], 0x1B, "Should end with ESC");
        assert_eq!(buf[len - 1], 0x5C, "Should end with backslash (ST)");

        // Verify the full expected format
        // DCS 2 $ u 9/17/25/33/41/49/57/65/73 ST
        let expected_data = "9/17/25/33/41/49/57/65/73";
        assert!(
            output.contains(expected_data),
            "Should contain expected data format, got: {output:?}",
        );

        // Verify length is reasonable
        assert!(len > 20, "Encoded length should be substantial");
    }

    #[test]
    fn test_cursor_style_report_encoding() {
        // Test encoding of DECRQSS cursor style response
        // Format: DCS 1 $ r Ps SP q ST
        //
        // This test creates a report with steady block cursor style (2)
        //
        // Expected output: DCS 1 $ r 2 q ST
        let report = CursorStyleReport {
            style: CursorStyleReportData(CursorStyle::SteadyBlock),
        };

        let mut buf = Vec::new();
        let len = report.encode_ansi_into(&mut buf).unwrap();

        // Expected format: ESC P 1 $ r <data> ESC \
        // ESC = 0x1B, P = 0x50, 1 = 0x31 (param), $ = 0x24, r = 0x72, \ = 0x5C
        assert_eq!(buf[0], 0x1B, "Should start with ESC");
        assert_eq!(buf[1], 0x50, "Should have P (DCS)");
        assert_eq!(buf[2], b'1', "Should have param '1'");
        assert_eq!(buf[3], 0x24, "Should have $ (intermediate)");
        assert_eq!(buf[4], 0x72, "Should have r (final byte)");

        // Check that the sequence contains the expected data
        let output = String::from_utf8_lossy(&buf);
        assert!(
            output.contains("2 q"),
            "Should contain cursor style '2 q', got: {output:?}"
        );

        // Check that it ends with ST (ESC \)
        assert_eq!(buf[len - 2], 0x1B, "Should end with ESC");
        assert_eq!(buf[len - 1], 0x5C, "Should end with backslash (ST)");
    }

    #[test]
    fn test_cursor_style_report_data_parsing() {
        use vtansi::parse::TryFromAnsi;

        // Test parsing various cursor style data formats
        let test_cases = [
            (b"0 q".as_slice(), CursorStyle::Default),
            (b"1 q".as_slice(), CursorStyle::BlinkingBlock),
            (b"2 q".as_slice(), CursorStyle::SteadyBlock),
            (b"3 q".as_slice(), CursorStyle::BlinkingUnderline),
            (b"4 q".as_slice(), CursorStyle::SteadyUnderline),
            (b"5 q".as_slice(), CursorStyle::BlinkingBar),
            (b"6 q".as_slice(), CursorStyle::SteadyBar),
        ];

        for (input, expected) in test_cases {
            let result = CursorStyleReportData::try_from_ansi(input).unwrap();
            assert_eq!(
                result.0,
                expected,
                "Failed to parse {:?}",
                String::from_utf8_lossy(input)
            );
        }

        // Test error cases
        assert!(
            CursorStyleReportData::try_from_ansi(b"2q").is_err(),
            "Should fail without space"
        );
        assert!(
            CursorStyleReportData::try_from_ansi(b"2 x").is_err(),
            "Should fail with wrong suffix"
        );
        assert!(
            CursorStyleReportData::try_from_ansi(b"7 q").is_err(),
            "Should fail with invalid style value"
        );
        assert!(
            CursorStyleReportData::try_from_ansi(b"q").is_err(),
            "Should fail with too short input"
        );
    }

    #[test]
    fn test_clear_tab_stop_encoding() {
        let clear = ClearTabStop;

        let mut buf = Vec::new();
        clear.encode_ansi_into(&mut buf).unwrap();
        let encoded = String::from_utf8(buf).unwrap();

        assert_eq!(encoded, "\x1b[g");
    }

    #[test]
    fn test_clear_all_tab_stops_encoding() {
        let clear = ClearAllTabStops;

        let mut buf = Vec::new();
        clear.encode_ansi_into(&mut buf).unwrap();
        let encoded = String::from_utf8(buf).unwrap();

        assert_eq!(encoded, "\x1b[3g");
    }

    #[test]
    fn test_character_position_absolute_encoding() {
        let hpa = CharacterPositionAbsolute(25);

        let mut buf = Vec::new();
        hpa.encode_ansi_into(&mut buf).unwrap();
        let encoded = String::from_utf8(buf).unwrap();

        assert_eq!(encoded, "\x1b[25`");
    }

    #[test]
    fn test_horizontal_vertical_position_encoding() {
        let hvp = HorizontalVerticalPosition { row: 10, col: 20 };

        let mut buf = Vec::new();
        hvp.encode_ansi_into(&mut buf).unwrap();
        let encoded = String::from_utf8(buf).unwrap();

        assert_eq!(encoded, "\x1b[10;20f");
    }

    #[test]
    fn test_horizontal_vertical_position_default() {
        let hvp = HorizontalVerticalPosition::default();

        assert_eq!(hvp.row, 1);
        assert_eq!(hvp.col, 1);

        let mut buf = Vec::new();
        hvp.encode_ansi_into(&mut buf).unwrap();
        let encoded = String::from_utf8(buf).unwrap();

        assert_eq!(encoded, "\x1b[1;1f");
    }
}
