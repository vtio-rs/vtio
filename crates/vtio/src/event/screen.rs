//! Screen and line erase commands.

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

#[cfg(test)]
mod tests {
    use super::*;
    use vtansi::AnsiEncode;

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
