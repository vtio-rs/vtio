//! Screen and line erase commands.

use crate::event::common::{Coords, Rect};

/// Erase Display Below (`ED`).
///
/// *Sequence*: `CSI Ps J` where `Ps` = 0 (default)
///
/// Erase from cursor position (inclusive) to the end of the screen.
///
/// Erases all cells from the cursor position to the end of the screen,
/// including the cell at the cursor position. This includes all cells on
/// the current line from the cursor to the end, and all cells on all lines
/// below the cursor line.
///
/// The erased cells are replaced with spaces using the current SGR
/// attributes (background color, etc.).
///
/// Does not move the cursor.
///
/// See <https://terminalguide.namepad.de/seq/csi_cj/> for
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
#[vtansi(csi, finalbyte = 'J')]
pub struct EraseDisplayBelow;

/// Erase Display Above (`ED`).
///
/// *Sequence*: `CSI 1 J`
///
/// Erase from the beginning of the screen to cursor position (inclusive).
///
/// Erases all cells from the beginning of the screen to the cursor position,
/// including the cell at the cursor position. This includes all cells on all
/// lines above the cursor line, and all cells from the beginning of the
/// current line to the cursor position.
///
/// The erased cells are replaced with spaces using the current SGR
/// attributes (background color, etc.).
///
/// Does not move the cursor.
///
/// See <https://terminalguide.namepad.de/seq/csi_cj/> for
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
#[vtansi(csi, params = ["1"], finalbyte = 'J')]
pub struct EraseDisplayAbove;

/// Erase Display Complete (`ED`).
///
/// *Sequence*: `CSI 2 J`
///
/// Erase the entire screen.
///
/// Erases all cells on the screen. The cursor position does not change.
///
/// Cells are cleared by replacing their contents with spaces, using the
/// current SGR attributes for the background color.
///
/// Note: This does not clear the scrollback buffer. Use
/// [`EraseDisplayScrollback`] for that. In some terminals (e.g., xterm with
/// specific options), this may scroll the screen content into the scrollback
/// buffer before clearing.
///
/// See <https://terminalguide.namepad.de/seq/csi_cj/> for
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
#[vtansi(csi, params = ["2"], finalbyte = 'J')]
pub struct EraseDisplayComplete;

/// Erase Display Scroll-back (`ED`).
///
/// *Sequence*: `CSI 3 J`
///
/// Erase the scrollback buffer.
///
/// This is an extended command that clears the terminal's scrollback buffer
/// (the off-screen history of previously displayed content). The visible
/// screen content is not affected.
///
/// This sequence is not supported by all terminals (notably not by urxvt).
///
/// Does not move the cursor.
///
/// See <https://terminalguide.namepad.de/seq/csi_cj/> for
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
#[vtansi(csi, params = ["3"], finalbyte = 'J')]
pub struct EraseDisplayScrollback;

/// Erase Line Right (`EL`).
///
/// *Sequence*: `CSI Ps K` where `Ps` = 0 (default)
///
/// Erase from cursor position (inclusive) to the end of the line.
///
/// Erases all cells from the cursor position to the end of the current line,
/// including the cell at the cursor position. The cursor position does not
/// change.
///
/// Cells are cleared by replacing their contents with spaces, using the
/// current SGR attributes for the background color.
///
/// See <https://terminalguide.namepad.de/seq/csi_ck/> for
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
#[vtansi(csi, finalbyte = 'K')]
pub struct EraseLineRight;

/// Erase Line Left (`EL`).
///
/// *Sequence*: `CSI 1 K`
///
/// Erase from the beginning of the line to cursor position (inclusive).
///
/// Erases all cells from the beginning of the current line to the cursor
/// position, including the cell at the cursor position. The cursor position
/// does not change.
///
/// Cells are cleared by replacing their contents with spaces, using the
/// current SGR attributes for the background color.
///
/// See <https://terminalguide.namepad.de/seq/csi_ck/> for
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
#[vtansi(csi, params = ["1"], finalbyte = 'K')]
pub struct EraseLineLeft;

/// Erase Line Complete (`EL`).
///
/// *Sequence*: `CSI 2 K`
///
/// Erase the entire line.
///
/// Erases all cells on the current line. The cursor position does not
/// change.
///
/// Cells are cleared by replacing their contents with spaces, using the
/// current SGR attributes for the background color.
///
/// See <https://terminalguide.namepad.de/seq/csi_ck/> for
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
#[vtansi(csi, params = ["2"], finalbyte = 'K')]
pub struct EraseLineComplete;

/// Selective Erase Display Below (`DECSED`).
///
/// *Sequence*: `CSI ? Ps J` where `Ps` = 0 (default)
///
/// Erase from cursor position (inclusive) to the end of the screen,
/// preserving protected cells.
///
/// Like [`EraseDisplayBelow`], but does not erase cells marked with
/// protected state. Protected cells retain their content while unprotected
/// cells are replaced with spaces using the current SGR attributes.
///
/// The cursor position does not change.
///
/// See <https://terminalguide.namepad.de/seq/csi_cj__p/> for
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
#[vtansi(csi, private = '?', finalbyte = 'J')]
pub struct SelectiveEraseDisplayBelow;

/// Selective Erase Display Above (`DECSED`).
///
/// *Sequence*: `CSI ? 1 J`
///
/// Erase from the beginning of the screen to cursor position (inclusive),
/// preserving protected cells.
///
/// Like [`EraseDisplayAbove`], but does not erase cells marked with
/// protected state. Protected cells retain their content while unprotected
/// cells are replaced with spaces using the current SGR attributes.
///
/// The cursor position does not change.
///
/// See <https://terminalguide.namepad.de/seq/csi_cj__p/> for
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
#[vtansi(csi, private = '?', params = ["1"], finalbyte = 'J')]
pub struct SelectiveEraseDisplayAbove;

/// Selective Erase Display Complete (`DECSED`).
///
/// *Sequence*: `CSI ? 2 J`
///
/// Erase the entire screen, preserving protected cells.
///
/// Like [`EraseDisplayComplete`], but does not erase cells marked with
/// protected state. Protected cells retain their content while unprotected
/// cells are replaced with spaces using the current SGR attributes.
///
/// The cursor position does not change.
///
/// See <https://terminalguide.namepad.de/seq/csi_cj__p/> for
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
#[vtansi(csi, private = '?', params = ["2"], finalbyte = 'J')]
pub struct SelectiveEraseDisplayComplete;

/// Selective Erase Line Right (`DECSEL`).
///
/// *Sequence*: `CSI ? Ps K` where `Ps` = 0 (default)
///
/// Erase from cursor position (inclusive) to the end of the line,
/// preserving protected cells.
///
/// Like [`EraseLineRight`], but does not erase cells marked with protected
/// state. Protected cells retain their content while unprotected cells are
/// replaced with spaces using the current SGR attributes.
///
/// The cursor position does not change.
///
/// See <https://terminalguide.namepad.de/seq/csi_ck__p/> for
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
#[vtansi(csi, private = '?', finalbyte = 'K')]
pub struct SelectiveEraseLineRight;

/// Selective Erase Line Left (`DECSEL`).
///
/// *Sequence*: `CSI ? 1 K`
///
/// Erase from the beginning of the line to cursor position (inclusive),
/// preserving protected cells.
///
/// Like [`EraseLineLeft`], but does not erase cells marked with protected
/// state. Protected cells retain their content while unprotected cells are
/// replaced with spaces using the current SGR attributes.
///
/// The cursor position does not change.
///
/// See <https://terminalguide.namepad.de/seq/csi_ck__p/> for
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
#[vtansi(csi, private = '?', params = ["1"], finalbyte = 'K')]
pub struct SelectiveEraseLineLeft;

/// Selective Erase Line Complete (`DECSEL`).
///
/// *Sequence*: `CSI ? 2 K`
///
/// Erase the entire line, preserving protected cells.
///
/// Like [`EraseLineComplete`], but does not erase cells marked with
/// protected state. Protected cells retain their content while unprotected
/// cells are replaced with spaces using the current SGR attributes.
///
/// The cursor position does not change.
///
/// See <https://terminalguide.namepad.de/seq/csi_ck__p/> for
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
#[vtansi(csi, private = '?', params = ["2"], finalbyte = 'K')]
pub struct SelectiveEraseLineComplete;

/// Insert Line (`IL`).
///
/// *Sequence*: `CSI Ps L`
///
/// Insert `amount` lines at the current cursor row.
///
/// The contents of the line at the current cursor row and below (to the
/// bottom-most line in the scrolling region) are shifted down by `amount`
/// lines. The contents of the `amount` bottom-most lines in the scroll
/// region are lost.
///
/// If the current cursor position is outside of the current scroll region,
/// it does nothing.
///
/// If `amount` is greater than the remaining number of lines in the
/// scrolling region, it is adjusted down.
///
/// In left and right margin mode, the margins are respected; lines are only
/// scrolled in the scroll region.
///
/// All cleared space is colored according to the current SGR state.
///
/// This unsets the pending wrap state without wrapping.
///
/// Moves the cursor to the left margin.
///
/// See <https://terminalguide.namepad.de/seq/csi_cl/> for
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
#[vtansi(csi, finalbyte = 'L')]
pub struct InsertLine(pub u16);

/// Delete Line (`DL`).
///
/// *Sequence*: `CSI Ps M`
///
/// Remove `amount` lines from the current cursor row down.
///
/// The remaining lines to the bottom margin are shifted up and space from
/// the bottom margin up is filled with empty lines.
///
/// If the current cursor position is outside of the current scroll region,
/// it does nothing.
///
/// If `amount` is greater than the remaining number of lines in the
/// scrolling region, it is adjusted down.
///
/// In left and right margin mode, the margins are respected; lines are only
/// scrolled in the scroll region.
///
/// All cleared space is colored according to the current SGR state.
///
/// This unsets the pending wrap state without wrapping.
///
/// Moves the cursor to the left margin.
///
/// See <https://terminalguide.namepad.de/seq/csi_cm/> for
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
#[vtansi(csi, finalbyte = 'M')]
pub struct DeleteLine(pub u16);

/// Delete Character (`DCH`).
///
/// *Sequence*: `CSI Ps P`
///
/// Remove `amount` characters from the current cursor position to the right.
///
/// The remaining characters are shifted to the left and space from the right
/// margin is filled with spaces.
///
/// If the current cursor column is not between the left and right margin, it
/// does nothing.
///
/// If `amount` is greater than the remaining number of characters in the
/// scrolling region, it is adjusted down.
///
/// In left and right margin mode, the margins are respected; characters are
/// only scrolled in the scroll region.
///
/// All newly cleared space starting from the right margin is colored
/// according to the current SGR state.
///
/// Does not change the cursor position.
///
/// This unsets the pending wrap state without wrapping.
///
/// See <https://terminalguide.namepad.de/seq/csi_cp/> for
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
#[vtansi(csi, finalbyte = 'P')]
pub struct DeleteCharacter(pub u16);

/// Insert Column (`DECIC`).
///
/// *Sequence*: `CSI Ps ' }`
///
/// Insert `amount` columns at the current cursor column.
///
/// Inserts `amount` columns (over the whole height of the current scrolling
/// region) from the current cursor column. The contents of the column at the
/// current cursor column and the columns to its right (to the right-most
/// column in the scrolling region) are shifted right by `amount` columns.
/// The contents of the `amount` right-most columns in the scroll region are
/// lost.
///
/// If the current cursor position is outside of the current scroll region,
/// it does nothing.
///
/// If `amount` is greater than the remaining number of columns in the
/// scrolling region, it is adjusted down.
///
/// In left and right margin mode, the margins are respected; columns are
/// only scrolled in the scroll region.
///
/// All cleared space is colored according to the current SGR state.
///
/// The cursor position is not changed.
///
/// See <https://terminalguide.namepad.de/seq/csi_x7d_right_brace_t_tick/>
/// for terminal support specifics.
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
#[vtansi(csi, intermediate = "'", finalbyte = '}')]
pub struct InsertColumn(pub u16);

/// Delete Column (`DECDC`).
///
/// *Sequence*: `CSI Ps ' ~`
///
/// Remove `amount` columns from the current cursor column to the right.
///
/// Removes `amount` columns (over the whole height of the current scrolling
/// region) from the current cursor column to the right. The remaining
/// columns to the right are shifted left and space from the right margin is
/// filled with empty cells.
///
/// If the current cursor position is outside of the current scroll region,
/// it does nothing.
///
/// If `amount` is greater than the remaining number of columns in the
/// scrolling region, it is adjusted down.
///
/// In left and right margin mode, the margins are respected; columns are
/// only scrolled in the scroll region.
///
/// All cleared space is colored according to the current SGR state.
///
/// The cursor position is not changed.
///
/// See <https://terminalguide.namepad.de/seq/csi_x7e_tilde_t_tick/> for
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
#[vtansi(csi, intermediate = "'", finalbyte = '~')]
pub struct DeleteColumn(pub u16);

/// Fill Screen with E (`DECALN`).
///
/// *Sequence*: `ESC # 8`
///
/// Fill the entire screen with the character 'E'.
///
/// This command is primarily used for screen alignment testing. It fills
/// the entire screen with the letter 'E' and moves the cursor to the top
/// left corner (1,1).
///
/// See <https://terminalguide.namepad.de/seq/a_esc_zhash_a8/> for
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
#[vtansi(esc, intermediate = "#", finalbyte = '8')]
pub struct FillScreenWithE;

/// Set Double Height Line Top Half (`DECDHL`).
///
/// *Sequence*: `ESC # 3`
///
/// Display double width and double height text (top half).
///
/// Sets a per-line attribute that allows displaying double height text.
/// For proper text display, two consecutive lines with identical text
/// content need to be output. The first line needs to be set with this
/// sequence, and the second line needs to be set with
/// [`SetDoubleHeightLineBottomHalf`].
///
/// If the line mode is switched from single width to this mode, the
/// number of columns is halved. If the cursor was in the second half of
/// the row, the cursor is moved to the new right-most column. The
/// columns no longer visible keep their contents and are revealed when
/// [`SetSingleWidthLine`] is set for the line.
///
/// In left and right margin mode, this control is ignored.
///
/// See <https://terminalguide.namepad.de/seq/a_esc_zhash_a3/> for
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
#[vtansi(esc, intermediate = "#", finalbyte = '3')]
pub struct SetDoubleHeightLineTopHalf;

/// Set Double Height Line Bottom Half (`DECDHL`).
///
/// *Sequence*: `ESC # 4`
///
/// Display double width and double height text (bottom half).
///
/// Sets a per-line attribute that allows displaying double height text.
/// For proper text display, two consecutive lines with identical text
/// content need to be output. The first line needs to be set with
/// [`SetDoubleHeightLineTopHalf`], and the second line needs to be set
/// with this sequence.
///
/// If the line mode is switched from single width to this mode, the
/// number of columns is halved. If the cursor was in the second half of
/// the row, the cursor is moved to the new right-most column. The
/// columns no longer visible keep their contents and are revealed when
/// [`SetSingleWidthLine`] is set for the line.
///
/// In left and right margin mode, this control is ignored.
///
/// See <https://terminalguide.namepad.de/seq/a_esc_zhash_a4/> for
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
#[vtansi(esc, intermediate = "#", finalbyte = '4')]
pub struct SetDoubleHeightLineBottomHalf;

/// Set Single Width Line (`DECSWL`).
///
/// *Sequence*: `ESC # 5`
///
/// Reset a line to normal single width and single height display mode.
///
/// This undoes the effect of [`SetDoubleHeightLineTopHalf`],
/// [`SetDoubleHeightLineBottomHalf`], and [`SetDoubleWidthLine`]. The
/// displayable columns are restored and the previously hidden text is
/// revealed.
///
/// See <https://terminalguide.namepad.de/seq/a_esc_zhash_a5/> for
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
#[vtansi(esc, intermediate = "#", finalbyte = '5')]
pub struct SetSingleWidthLine;

/// Set Double Width Line (`DECDWL`).
///
/// *Sequence*: `ESC # 6`
///
/// Display double width and single height text.
///
/// Sets a per-line attribute that allows displaying double width text.
/// Text is displayed using double the normal amount of cell spaces per
/// character.
///
/// If the line mode is switched from single width to this mode, the
/// number of columns is halved. If the cursor was in the second half of
/// the row, the cursor is moved to the new right-most column. The
/// columns no longer visible keep their contents and are revealed when
/// [`SetSingleWidthLine`] is set for the line.
///
/// In left and right margin mode, this control is ignored.
///
/// See <https://terminalguide.namepad.de/seq/a_esc_zhash_a6/> for
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
#[vtansi(esc, intermediate = "#", finalbyte = '6')]
pub struct SetDoubleWidthLine;

/// Insert Character (`ICH`).
///
/// *Sequence*: `CSI Ps @`
///
/// Insert `amount` blank character(s) at the current cursor position.
///
/// Inserts `amount` blank (space) characters at the current cursor position.
/// The character at the cursor and all characters to its right are shifted
/// right by `amount` positions. Characters shifted past the right margin
/// are lost.
///
/// If the current cursor column is not between the left and right margin, it
/// does nothing.
///
/// If `amount` is greater than the remaining number of characters in the
/// scrolling region, it is adjusted down.
///
/// In left and right margin mode, the margins are respected; characters are
/// only scrolled in the scroll region.
///
/// All newly inserted blank space is colored according to the current SGR
/// state.
///
/// Does not change the cursor position.
///
/// This unsets the pending wrap state without wrapping.
///
/// See <https://terminalguide.namepad.de/seq/csi_x40_at/> for
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
#[vtansi(csi, finalbyte = '@')]
pub struct InsertCharacter(pub u16);

/// Erase Character (`ECH`).
///
/// *Sequence*: `CSI Ps X`
///
/// Erase `amount` character(s) at the current cursor position.
///
/// Erases `amount` characters starting at the current cursor position.
/// Characters are erased by replacing them with spaces using the current
/// SGR attributes (background color, etc.).
///
/// Unlike [`DeleteCharacter`], this does not shift characters; it only
/// replaces them with blanks in place.
///
/// If `amount` extends beyond the right margin, only characters up to the
/// right margin are erased.
///
/// Does not change the cursor position.
///
/// This unsets the pending wrap state without wrapping.
///
/// See <https://terminalguide.namepad.de/seq/csi_cx/> for
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
#[vtansi(csi, finalbyte = 'X')]
pub struct EraseCharacter(pub u16);

/// Repeat Character (`REP`).
///
/// *Sequence*: `CSI Ps b`
///
/// Repeat the preceding graphic character `amount` times.
///
/// Repeats the most recently printed graphic character `amount` times at
/// the current cursor position. The cursor advances as if the character
/// were typed that many times.
///
/// If no graphic character has been printed since the terminal was reset
/// or since the last control function, the behavior is undefined (typically
/// nothing happens or a space is repeated).
///
/// The repeated character uses the current SGR attributes, which may differ
/// from the attributes used when the character was originally printed.
///
/// This is commonly used by applications to efficiently output repeated
/// characters without sending the same byte multiple times.
///
/// See <https://terminalguide.namepad.de/seq/csi_sb/> for
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
#[vtansi(csi, finalbyte = 'b')]
pub struct RepeatCharacter(pub u16);

// =============================================================================
// Rectangular Area Operations
// =============================================================================

/// Fill Rectangular Area (`DECFRA`).
///
/// *Sequence*: `CSI Pc ; Pt ; Pl ; Pb ; Pr $ x`
///
/// Fill a rectangular area with a specified character.
///
/// Fills the rectangular area defined by `area` with the character specified
/// by `character` (a decimal ASCII code). The fill operation respects the
/// current character protection attribute (DECSCA).
///
/// # Parameters
///
/// - `character`: ASCII code of the character to fill with (e.g., 32 for space)
/// - `area`: The rectangular area to fill
///
/// # Example
///
/// ```
/// use vtio::event::screen::FillRectangle;
/// use vtio::event::Rect;
/// use vtansi::AnsiEncode;
///
/// // Fill a 5x10 rectangle starting at row 1, column 1 with spaces (ASCII 32)
/// let fill = FillRectangle {
///     character: 32,
///     area: Rect::new(1, 1, 5, 10),
/// };
/// let mut buf = Vec::new();
/// fill.encode_ansi_into(&mut buf).unwrap();
/// assert_eq!(&buf, b"\x1b[32;1;1;5;10$x");
/// ```
///
/// # See Also
///
/// - [`EraseRectangle`] - Erase (blank) a rectangular area
/// - [`SelectiveEraseRectangle`] - Selective erase respecting protection
/// - <https://vt100.net/docs/vt510-rm/DECFRA.html>
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, vtansi::derive::AnsiOutput,
)]
#[vtansi(csi, intermediate = "$", finalbyte = 'x')]
pub struct FillRectangle {
    /// ASCII code of the character to fill with.
    pub character: u16,
    /// The rectangular area to fill.
    #[vtansi(flatten)]
    pub area: Rect,
}

impl FillRectangle {
    /// Create a new fill rectangle command.
    #[must_use]
    pub const fn new(character: u16, area: Rect) -> Self {
        Self { character, area }
    }
}

/// Erase Rectangular Area (`DECERA`).
///
/// *Sequence*: `CSI Pt ; Pl ; Pb ; Pr $ z`
///
/// Erase (fill with spaces) a rectangular area.
///
/// Erases the rectangular area by filling it with space characters using
/// the current SGR attributes. This operation does not respect character
/// protection (DECSCA).
///
/// # Example
///
/// ```
/// use vtio::event::screen::EraseRectangle;
/// use vtio::event::Rect;
/// use vtansi::AnsiEncode;
///
/// // Erase a rectangle from row 1, col 1 to row 10, col 80
/// let erase = EraseRectangle::new(Rect::new(1, 1, 10, 80));
/// let mut buf = Vec::new();
/// erase.encode_ansi_into(&mut buf).unwrap();
/// assert_eq!(&buf, b"\x1b[1;1;10;80$z");
/// ```
///
/// # See Also
///
/// - [`FillRectangle`] - Fill with a specific character
/// - [`SelectiveEraseRectangle`] - Selective erase respecting protection
/// - <https://vt100.net/docs/vt510-rm/DECERA.html>
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, vtansi::derive::AnsiOutput,
)]
#[vtansi(csi, intermediate = "$", finalbyte = 'z')]
pub struct EraseRectangle(
    /// The rectangular area to erase.
    #[vtansi(flatten)]
    pub Rect,
);

impl EraseRectangle {
    /// Create a new erase rectangle command.
    #[must_use]
    pub const fn new(area: Rect) -> Self {
        Self(area)
    }

    /// Get the rectangular area.
    #[must_use]
    pub const fn area(&self) -> Rect {
        self.0
    }
}

/// Selective Erase Rectangular Area (`DECSERA`).
///
/// *Sequence*: `CSI Pt ; Pl ; Pb ; Pr $ {`
///
/// Selectively erase a rectangular area, respecting character protection.
///
/// Similar to [`EraseRectangle`], but only erases characters that are not
/// protected by the character protection attribute (DECSCA). Protected
/// characters within the rectangle are left unchanged.
///
/// # Example
///
/// ```
/// use vtio::event::screen::SelectiveEraseRectangle;
/// use vtio::event::Rect;
/// use vtansi::AnsiEncode;
///
/// // Selective erase a rectangle respecting character protection
/// let erase = SelectiveEraseRectangle::new(Rect::new(5, 10, 20, 70));
/// let mut buf = Vec::new();
/// erase.encode_ansi_into(&mut buf).unwrap();
/// assert_eq!(&buf, b"\x1b[5;10;20;70${");
/// ```
///
/// # See Also
///
/// - [`EraseRectangle`] - Unconditional erase
/// - [`FillRectangle`] - Fill with a specific character
/// - <https://vt100.net/docs/vt510-rm/DECSERA.html>
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, vtansi::derive::AnsiOutput,
)]
#[vtansi(csi, intermediate = "$", finalbyte = '{')]
pub struct SelectiveEraseRectangle(
    /// The rectangular area to selectively erase.
    #[vtansi(flatten)]
    pub Rect,
);

impl SelectiveEraseRectangle {
    /// Create a new selective erase rectangle command.
    #[must_use]
    pub const fn new(area: Rect) -> Self {
        Self(area)
    }

    /// Get the rectangular area.
    #[must_use]
    pub const fn area(&self) -> Rect {
        self.0
    }
}

/// Copy Rectangular Area (`DECCRA`).
///
/// *Sequence*: `CSI Pts ; Pls ; Pbs ; Prs ; Pps ; Ptd ; Pld ; Ppd $ v`
///
/// Copy a rectangular area from one location to another.
///
/// Copies the contents of the source rectangle to the destination location.
/// The source and destination can be on different pages if the terminal
/// supports multiple pages.
///
/// # Parameters
///
/// - `source`: The source rectangular area to copy from
/// - `source_page`: The page number of the source (1-based, typically 1)
/// - `dest_top`: Top line of the destination
/// - `dest_left`: Left column of the destination
/// - `dest_page`: The page number of the destination (1-based, typically 1)
///
/// # Example
///
/// ```
/// use vtio::event::screen::CopyRectangle;
/// use vtio::event::Rect;
/// use vtansi::AnsiEncode;
///
/// // Copy rectangle from (1,1)-(10,20) to (15,1) on page 1
/// let copy = CopyRectangle {
///     source: Rect::new(1, 1, 10, 20),
///     source_page: 1,
///     dest: vtio::event::Coords::new(15, 1),
///     dest_page: 1,
/// };
/// let mut buf = Vec::new();
/// copy.encode_ansi_into(&mut buf).unwrap();
/// assert_eq!(&buf, b"\x1b[1;1;10;20;1;15;1;1$v");
/// ```
///
/// # See Also
///
/// - [`FillRectangle`] - Fill a rectangular area
/// - [`EraseRectangle`] - Erase a rectangular area
/// - <https://vt100.net/docs/vt510-rm/DECCRA.html>
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, vtansi::derive::AnsiOutput,
)]
#[vtansi(csi, intermediate = "$", finalbyte = 'v')]
pub struct CopyRectangle {
    /// The source rectangular area.
    #[vtansi(flatten)]
    pub source: Rect,
    /// Page number of the source (1-based).
    pub source_page: u16,
    /// Destination position (top-left corner).
    #[vtansi(flatten)]
    pub dest: Coords,
    /// Page number of the destination (1-based).
    pub dest_page: u16,
}

impl CopyRectangle {
    /// Create a new copy rectangle command.
    ///
    /// Uses page 1 for both source and destination.
    #[must_use]
    pub const fn new(source: Rect, dest_top: u16, dest_left: u16) -> Self {
        Self {
            source,
            source_page: 1,
            dest: Coords::new(dest_top, dest_left),
            dest_page: 1,
        }
    }

    /// Create a new copy rectangle command with explicit page numbers.
    #[must_use]
    pub const fn with_pages(
        source: Rect,
        source_page: u16,
        dest_top: u16,
        dest_left: u16,
        dest_page: u16,
    ) -> Self {
        Self {
            source,
            source_page,
            dest: Coords::new(dest_top, dest_left),
            dest_page,
        }
    }
}

/// Request Checksum of Rectangular Area (`DECRQCRA`).
///
/// *Sequence*: `CSI Pid ; Pp ; Pt ; Pl ; Pb ; Pr * y`
///
/// Request the terminal to compute and report a checksum of the characters
/// in a rectangular area.
///
/// The terminal responds with a [`RectangularChecksumReport`] containing
/// the computed checksum.
///
/// # Parameters
///
/// - `request_id`: An identifier that will be echoed in the response
/// - `page`: Page number (1-based, typically 1)
/// - `area`: The rectangular area to checksum
///
/// # Example
///
/// ```
/// use vtio::event::screen::RequestRectangularChecksum;
/// use vtio::event::Rect;
/// use vtansi::AnsiEncode;
///
/// // Request checksum of entire screen (assuming 24x80)
/// let request = RequestRectangularChecksum {
///     request_id: 1,
///     page: 1,
///     area: Rect::new(1, 1, 24, 80),
/// };
/// let mut buf = Vec::new();
/// request.encode_ansi_into(&mut buf).unwrap();
/// assert_eq!(&buf, b"\x1b[1;1;1;1;24;80*y");
/// ```
///
/// # See Also
///
/// - [`RectangularChecksumReport`] - Response from the terminal
/// - <https://vt100.net/docs/vt510-rm/DECRQCRA.html>
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, vtansi::derive::AnsiOutput,
)]
#[vtansi(csi, intermediate = "*", finalbyte = 'y')]
pub struct RequestRectangularChecksum {
    /// Request identifier (echoed in response).
    pub request_id: u16,
    /// Page number (1-based).
    pub page: u16,
    /// The rectangular area to checksum.
    #[vtansi(flatten)]
    pub area: Rect,
}

impl RequestRectangularChecksum {
    /// Create a new checksum request.
    #[must_use]
    pub const fn new(request_id: u16, page: u16, area: Rect) -> Self {
        Self {
            request_id,
            page,
            area,
        }
    }
}

/// Rectangular Area Checksum Report (`DECCKSR`).
///
/// *Sequence*: `DCS Pid ! ~ D...D ST`
///
/// Response to [`RequestRectangularChecksum`] containing the computed
/// checksum of the requested rectangular area.
///
/// # Parameters
///
/// - `request_id`: The identifier from the original request
/// - `checksum`: The computed checksum as a 4-digit hexadecimal string
///
/// # Example
///
/// ```
/// use vtio::event::screen::{RectangularChecksumReport, RectangularChecksumData};
///
/// // A typical response might look like:
/// // DCS 1 ! ~ 3A2F ST
/// let report = RectangularChecksumReport {
///     request_id: 1,
///     checksum: RectangularChecksumData(0x3A2F),
/// };
/// assert_eq!(report.request_id, 1);
/// assert_eq!(report.checksum.0, 0x3A2F);
/// ```
///
/// # See Also
///
/// - [`RequestRectangularChecksum`] - Request this report
/// - <https://vt100.net/docs/vt510-rm/DECCKSR.html>
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, vtansi::derive::AnsiInput,
)]
#[vtansi(dcs, locate_all = "data", intermediate = "!", finalbyte = '~')]
pub struct RectangularChecksumReport {
    /// The request identifier from the original request.
    pub request_id: u16,
    /// The computed checksum.
    pub checksum: RectangularChecksumData,
}

/// Wrapper for parsing and encoding the checksum data in [`RectangularChecksumReport`].
///
/// The checksum is transmitted as a 4-character hexadecimal string in the DCS data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct RectangularChecksumData(pub u16);

impl vtansi::parse::TryFromAnsi<'_> for RectangularChecksumData {
    fn try_from_ansi(bytes: &[u8]) -> Result<Self, vtansi::ParseError> {
        // The format is "Pid ! ~ XXXX" where XXXX is a 4-digit hex checksum
        // By the time we get here, we just have the hex digits
        if bytes.is_empty() {
            return Ok(Self(0));
        }

        let hex_str = std::str::from_utf8(bytes).map_err(|_| {
            vtansi::ParseError::InvalidValue(
                "checksum must be valid UTF-8".to_string(),
            )
        })?;

        let checksum = u16::from_str_radix(hex_str, 16).map_err(|_| {
            vtansi::ParseError::InvalidValue(format!(
                "invalid checksum hex value: {hex_str}"
            ))
        })?;

        Ok(Self(checksum))
    }
}

impl vtansi::encode::AnsiEncode for RectangularChecksumData {
    fn encode_ansi_into<W: std::io::Write + ?Sized>(
        &self,
        writer: &mut W,
    ) -> Result<usize, vtansi::EncodeError> {
        // Encode as 4-digit uppercase hex
        let hex = format!("{:04X}", self.0);
        vtansi::encode::write_bytes_into(writer, hex.as_bytes())
    }
}

impl From<u16> for RectangularChecksumData {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl From<RectangularChecksumData> for u16 {
    fn from(value: RectangularChecksumData) -> Self {
        value.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vtansi::AnsiEncode;

    // =========================================================================
    // Rectangular Area Tests
    // =========================================================================

    #[test]
    fn test_fill_rectangle_encoding() {
        let fill = FillRectangle {
            character: 32, // space
            area: Rect::new(1, 1, 10, 80),
        };
        assert_eq!(fill.encode_ansi().unwrap(), b"\x1b[32;1;1;10;80$x");
    }

    #[test]
    fn test_fill_rectangle_new() {
        let area = Rect::new(5, 10, 15, 70);
        let fill = FillRectangle::new(u16::from(b'#'), area);
        assert_eq!(fill.character, 35);
        assert_eq!(fill.area, area);
    }

    #[test]
    fn test_erase_rectangle_encoding() {
        let erase = EraseRectangle(Rect::new(1, 1, 24, 80));
        assert_eq!(erase.encode_ansi().unwrap(), b"\x1b[1;1;24;80$z");
    }

    #[test]
    fn test_erase_rectangle_new() {
        let area = Rect::new(1, 1, 10, 20);
        let erase = EraseRectangle::new(area);
        assert_eq!(erase.area(), area);
    }

    #[test]
    fn test_selective_erase_rectangle_encoding() {
        let erase = SelectiveEraseRectangle(Rect::new(5, 10, 15, 70));
        assert_eq!(erase.encode_ansi().unwrap(), b"\x1b[5;10;15;70${");
    }

    #[test]
    fn test_selective_erase_rectangle_new() {
        let area = Rect::new(2, 3, 8, 9);
        let erase = SelectiveEraseRectangle::new(area);
        assert_eq!(erase.area(), area);
    }

    #[test]
    fn test_copy_rectangle_encoding() {
        let copy = CopyRectangle {
            source: Rect::new(1, 1, 10, 20),
            source_page: 1,
            dest: Coords::new(15, 1),
            dest_page: 1,
        };
        assert_eq!(copy.encode_ansi().unwrap(), b"\x1b[1;1;10;20;1;15;1;1$v");
    }

    #[test]
    fn test_copy_rectangle_new() {
        let source = Rect::new(1, 1, 10, 20);
        let copy = CopyRectangle::new(source, 15, 5);
        assert_eq!(copy.source, source);
        assert_eq!(copy.dest, Coords::new(15, 5));
        assert_eq!(copy.source_page, 1);
        assert_eq!(copy.dest_page, 1);
    }

    #[test]
    fn test_copy_rectangle_with_pages() {
        let source = Rect::new(1, 1, 10, 20);
        let copy = CopyRectangle::with_pages(source, 2, 15, 5, 3);
        assert_eq!(copy.source_page, 2);
        assert_eq!(copy.dest_page, 3);
    }

    #[test]
    fn test_request_rectangular_checksum_encoding() {
        let request = RequestRectangularChecksum {
            request_id: 42,
            page: 1,
            area: Rect::new(1, 1, 24, 80),
        };
        assert_eq!(request.encode_ansi().unwrap(), b"\x1b[42;1;1;1;24;80*y");
    }

    #[test]
    fn test_request_rectangular_checksum_new() {
        let area = Rect::new(1, 1, 24, 80);
        let request = RequestRectangularChecksum::new(1, 1, area);
        assert_eq!(request.request_id, 1);
        assert_eq!(request.page, 1);
        assert_eq!(request.area, area);
    }

    #[test]
    fn test_rectangular_checksum_data_encoding() {
        use vtansi::AnsiEncode;

        let data = RectangularChecksumData(0x3A2F);
        let mut buf = Vec::new();
        data.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(&buf, b"3A2F");

        // Test zero padding
        let data = RectangularChecksumData(0x00FF);
        let mut buf = Vec::new();
        data.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(&buf, b"00FF");
    }

    #[test]
    fn test_rectangular_checksum_data_parsing() {
        use vtansi::parse::TryFromAnsi;

        let data = RectangularChecksumData::try_from_ansi(b"3A2F").unwrap();
        assert_eq!(data.0, 0x3A2F);

        let data = RectangularChecksumData::try_from_ansi(b"00FF").unwrap();
        assert_eq!(data.0, 0x00FF);

        let data = RectangularChecksumData::try_from_ansi(b"").unwrap();
        assert_eq!(data.0, 0);
    }

    #[test]
    fn test_rectangular_checksum_data_conversion() {
        let data: RectangularChecksumData = 0x1234u16.into();
        assert_eq!(data.0, 0x1234);

        let value: u16 = data.into();
        assert_eq!(value, 0x1234);
    }

    // =========================================================================
    // Original Tests
    // =========================================================================

    #[test]
    fn test_insert_character_encoding() {
        assert_eq!(InsertCharacter(1).encode_ansi().unwrap(), b"\x1b[1@");
        assert_eq!(InsertCharacter(5).encode_ansi().unwrap(), b"\x1b[5@");
        assert_eq!(InsertCharacter(10).encode_ansi().unwrap(), b"\x1b[10@");
    }

    #[test]
    fn test_insert_character_default() {
        // Default parameter value of 1
        assert_eq!(InsertCharacter(1).encode_ansi().unwrap(), b"\x1b[1@");
    }

    #[test]
    fn test_erase_character_encoding() {
        assert_eq!(EraseCharacter(1).encode_ansi().unwrap(), b"\x1b[1X");
        assert_eq!(EraseCharacter(5).encode_ansi().unwrap(), b"\x1b[5X");
        assert_eq!(EraseCharacter(10).encode_ansi().unwrap(), b"\x1b[10X");
    }

    #[test]
    fn test_erase_character_default() {
        // Default parameter value of 1
        assert_eq!(EraseCharacter(1).encode_ansi().unwrap(), b"\x1b[1X");
    }

    #[test]
    fn test_repeat_character_encoding() {
        assert_eq!(RepeatCharacter(1).encode_ansi().unwrap(), b"\x1b[1b");
        assert_eq!(RepeatCharacter(5).encode_ansi().unwrap(), b"\x1b[5b");
        assert_eq!(RepeatCharacter(80).encode_ansi().unwrap(), b"\x1b[80b");
    }

    #[test]
    fn test_repeat_character_default() {
        // Default parameter value of 1
        assert_eq!(RepeatCharacter(1).encode_ansi().unwrap(), b"\x1b[1b");
    }
}
