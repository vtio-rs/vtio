//! Mouse event encoding formats.
//!
//! This module contains the different encoding formats used for mouse events
//! in terminal protocols:
//!
//! - **SGR format** (`SgrMouseEvent`): The preferred extended format using
//!   `ESC[<btn;col;rowM` for press and `ESC[<btn;col;rowm` for release.
//! - **urxvt format** (`UrxvtMouseEvent`): Uses `ESC[btn;col;rowM` with
//!   button code offset by 32.
//! - **Default/Multibyte format**: Uses `ESC[Mbtncolrow` with raw bytes or
//!   UTF-8 encoded values for extended coordinates.
//!
//! See <https://terminalguide.namepad.de/mouse/> for details.

use vtansi::{
    AnsiEncode, EncodeError, ParseError, RawByte, TryFromAnsi, TryFromAnsiIter,
    write_csi,
};

use super::{
    Coordinates, MouseEvent, MouseEventKind, MouseKeyModifiers,
    modifiers_from_button_code,
};

// ============================================================================
// MouseEvent AnsiEncode (forwards to SGR format)
// ============================================================================

impl AnsiEncode for MouseEvent {
    fn encode_ansi_into<W: std::io::Write + ?Sized>(
        &self,
        sink: &mut W,
    ) -> Result<usize, EncodeError> {
        // Encode as SGR format by default
        SgrMouseEventSeq(SgrMouseEvent(*self)).encode_ansi_into(sink)
    }
}

// ============================================================================
// SGR Mouse Event (CSI < btn ; col ; row M/m)
// ============================================================================

/// Mouse event in SGR (digits) reporting format.
///
/// *Sequence*: `CSI < Pb ; Px ; Py M` (press) / `CSI < Pb ; Px ; Py m` (release)
///
/// This structure parses mouse events using the SGR mouse reporting
/// format, which uses sequences like `ESC[<btn;col;row;M` for button
/// press/movement and `ESC[<btn;col;row;m` for button release.
///
/// The SGR format is the preferred extended coordinate format as it:
/// - Has no arbitrary range limit
/// - Uses a different final byte for release (`m`) vs press (`M`)
/// - Only contains ASCII characters
///
/// See <https://terminalguide.namepad.de/mode/p1006/> for terminal support.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct SgrMouseEvent(pub MouseEvent);

impl<'a> TryFromAnsiIter<'a> for SgrMouseEvent {
    fn try_from_ansi_iter<I>(params: &mut I) -> Result<Self, ParseError>
    where
        I: Iterator<Item = &'a [u8]>,
    {
        // Parse button code
        let btn_bytes = params.next().ok_or_else(|| {
            ParseError::InvalidValue(
                "SGR mouse: missing button code parameter".to_string(),
            )
        })?;
        let btn_code = <u16 as TryFromAnsi>::try_from_ansi(btn_bytes)?;

        // Parse column (1-based)
        let col_bytes = params.next().ok_or_else(|| {
            ParseError::InvalidValue(
                "SGR mouse: missing column parameter".to_string(),
            )
        })?;
        let column = <u16 as TryFromAnsi>::try_from_ansi(col_bytes)?;

        // Parse row (1-based)
        let row_bytes = params.next().ok_or_else(|| {
            ParseError::InvalidValue(
                "SGR mouse: missing row parameter".to_string(),
            )
        })?;
        let row = <u16 as TryFromAnsi>::try_from_ansi(row_bytes)?;

        // In SGR format, release is indicated by final byte 'm', not button code
        // The final byte handling is done in SgrMouseEventSeq
        // Here we parse assuming it's a press event; the Seq wrapper will fix it
        let kind = MouseEventKind::from_button_code(btn_code, false)?;
        let modifiers = MouseKeyModifiers(modifiers_from_button_code(btn_code));

        Ok(SgrMouseEvent(MouseEvent {
            kind,
            modifiers,
            coords: Coordinates { column, row },
        }))
    }
}

impl<'a> TryFromAnsi<'a> for SgrMouseEvent {
    #[inline]
    fn try_from_ansi(bytes: &'a [u8]) -> Result<Self, ParseError> {
        <SgrMouseEvent as TryFromAnsiIter>::try_from_ansi_iter(
            &mut bytes.split(|&c| c == b';'),
        )
    }
}

impl From<SgrMouseEvent> for MouseEvent {
    #[inline]
    fn from(sgr: SgrMouseEvent) -> Self {
        sgr.0
    }
}

impl AnsiEncode for SgrMouseEvent {
    fn encode_ansi_into<W: std::io::Write + ?Sized>(
        &self,
        sink: &mut W,
    ) -> Result<usize, EncodeError> {
        let event = &self.0;
        let btn_code =
            u16::from(event.kind) | u16::from(event.modifiers.bits());

        let mut written =
            <u16 as AnsiEncode>::encode_ansi_into(&btn_code, sink)?;
        written += vtansi::write_byte_into(sink, b';')?;
        written +=
            <u16 as AnsiEncode>::encode_ansi_into(&event.coords.column, sink)?;
        written += vtansi::write_byte_into(sink, b';')?;
        written +=
            <u16 as AnsiEncode>::encode_ansi_into(&event.coords.row, sink)?;

        Ok(written)
    }
}

/// SGR mouse event input sequence wrapper.
///
/// *Sequence*: `CSI < Pb ; Px ; Py M` (press) / `CSI < Pb ; Px ; Py m` (release)
///
/// This is the CSI sequence wrapper that hooks into the vtansi parsing system.
/// It handles the final byte ('M' for press, 'm' for release) and delegates
/// parameter parsing to `SgrMouseEvent`.
#[derive(
    Debug, PartialEq, Eq, Clone, Copy, Hash, vtansi::derive::AnsiInput,
)]
#[vtansi(csi, private = '<', into = MouseEvent, finalbyte = 'M' | 'm')]
pub struct SgrMouseEventSeq(#[vtansi(flatten)] pub SgrMouseEvent);

impl vtansi::AnsiFinalByte for SgrMouseEventSeq {
    #[inline]
    fn ansi_final_byte(&self) -> u8 {
        match self.0.0.kind {
            MouseEventKind::Up(_) => b'm',
            _ => b'M',
        }
    }
}

impl From<SgrMouseEventSeq> for MouseEvent {
    #[inline]
    fn from(seq: SgrMouseEventSeq) -> Self {
        seq.0.into()
    }
}

// ============================================================================
// urxvt Mouse Event (CSI btn ; col ; row M)
// ============================================================================

/// Mouse event in urxvt reporting format.
///
/// *Sequence*: `CSI Pb ; Px ; Py M`
///
/// This structure parses mouse events using the urxvt mouse reporting
/// format, which uses sequences like `ESC [ btn ; column ; row M`.
///
/// Unlike the SGR format:
/// - No private byte (`<`)
/// - Button code is offset by 32 (like the default format)
/// - Button release is indicated by button code 3, not by final byte
/// - Always uses final byte `M`
///
/// The urxvt format does not have an arbitrary range limit and only contains
/// ASCII characters, making it decodable with a standard UTF-8 decoder.
///
/// See <https://terminalguide.namepad.de/mode/p1015/> for terminal support.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct UrxvtMouseEvent(pub MouseEvent);

impl<'a> TryFromAnsiIter<'a> for UrxvtMouseEvent {
    fn try_from_ansi_iter<I>(params: &mut I) -> Result<Self, ParseError>
    where
        I: Iterator<Item = &'a [u8]>,
    {
        // Parse button code (offset by 32)
        let btn_bytes = params.next().ok_or_else(|| {
            ParseError::InvalidValue(
                "urxvt mouse: missing button code parameter".to_string(),
            )
        })?;
        let btn_with_offset = <u16 as TryFromAnsi>::try_from_ansi(btn_bytes)?;
        // Remove the offset of 32 to get the actual button code
        let btn_code = btn_with_offset.saturating_sub(32);

        // Parse column (1-based)
        let col_bytes = params.next().ok_or_else(|| {
            ParseError::InvalidValue(
                "urxvt mouse: missing column parameter".to_string(),
            )
        })?;
        let column = <u16 as TryFromAnsi>::try_from_ansi(col_bytes)?;

        // Parse row (1-based)
        let row_bytes = params.next().ok_or_else(|| {
            ParseError::InvalidValue(
                "urxvt mouse: missing row parameter".to_string(),
            )
        })?;
        let row = <u16 as TryFromAnsi>::try_from_ansi(row_bytes)?;

        // In urxvt format, release is indicated by button code 3 (like default format)
        // not by a different final byte like in SGR format
        let kind = MouseEventKind::from_button_code(btn_code, false)?;
        let modifiers = MouseKeyModifiers(modifiers_from_button_code(btn_code));

        Ok(UrxvtMouseEvent(MouseEvent {
            kind,
            modifiers,
            coords: Coordinates { column, row },
        }))
    }
}

impl<'a> TryFromAnsi<'a> for UrxvtMouseEvent {
    #[inline]
    fn try_from_ansi(bytes: &'a [u8]) -> Result<Self, ParseError> {
        <UrxvtMouseEvent as TryFromAnsiIter>::try_from_ansi_iter(
            &mut bytes.split(|&c| c == b';'),
        )
    }
}

impl From<UrxvtMouseEvent> for MouseEvent {
    #[inline]
    fn from(urxvt: UrxvtMouseEvent) -> Self {
        urxvt.0
    }
}

impl AnsiEncode for UrxvtMouseEvent {
    fn encode_ansi_into<W: std::io::Write + ?Sized>(
        &self,
        sink: &mut W,
    ) -> Result<usize, EncodeError> {
        let event = &self.0;
        // urxvt format uses button code offset by 32
        let btn_code =
            (u16::from(event.kind) | u16::from(event.modifiers.bits())) + 32;

        let mut written =
            <u16 as AnsiEncode>::encode_ansi_into(&btn_code, sink)?;
        written += vtansi::write_byte_into(sink, b';')?;
        written +=
            <u16 as AnsiEncode>::encode_ansi_into(&event.coords.column, sink)?;
        written += vtansi::write_byte_into(sink, b';')?;
        written +=
            <u16 as AnsiEncode>::encode_ansi_into(&event.coords.row, sink)?;

        Ok(written)
    }
}

/// urxvt mouse event input sequence wrapper.
///
/// *Sequence*: `CSI Pb ; Px ; Py M`
///
/// This is the CSI sequence wrapper that hooks into the vtansi parsing system.
/// It delegates parameter parsing to `UrxvtMouseEvent`.
#[derive(
    Debug, PartialEq, Eq, Clone, Copy, Hash, vtansi::derive::AnsiInput,
)]
#[vtansi(csi, into = MouseEvent, finalbyte = 'M')]
pub struct UrxvtMouseEventSeq(#[vtansi(flatten)] pub UrxvtMouseEvent);

impl From<UrxvtMouseEventSeq> for MouseEvent {
    #[inline]
    fn from(seq: UrxvtMouseEventSeq) -> Self {
        seq.0.into()
    }
}

// ============================================================================
// Default/Multibyte Mouse Event (CSI M btn col row)
// ============================================================================

/// Decode a single mouse coordinate value from UTF-8 encoded bytes.
///
/// This handles both the default format (single byte for values < 96)
/// and the UTF-8 multibyte format (p1005) where values >= 96 are encoded
/// as UTF-8 codepoints with value + 32.
///
/// Returns the decoded value (with offset 32 subtracted) and the number
/// of bytes consumed, or None if the input is invalid or represents
/// an out-of-range marker (NUL byte).
fn decode_mouse_value(bytes: &[u8]) -> Option<(u16, usize)> {
    if bytes.is_empty() {
        return None;
    }

    let first = bytes[0];

    // NUL is used as out-of-range marker
    if first == 0 {
        return None;
    }

    // Single byte (ASCII): values 1-127
    if first < 0x80 {
        // Value is encoded as byte = actual_value + 32
        return Some((u16::from(first.saturating_sub(32)), 1));
    }

    // Two-byte UTF-8 sequence: 110xxxxx 10xxxxxx
    // This encodes codepoints U+0080 to U+07FF (128 to 2047)
    // Which corresponds to values 96 to 2015 (after subtracting 32)
    if first & 0xE0 == 0xC0 && bytes.len() >= 2 {
        let second = bytes[1];
        if second & 0xC0 == 0x80 {
            // Decode UTF-8: ((first & 0x1F) << 6) | (second & 0x3F)
            let codepoint =
                (u16::from(first & 0x1F) << 6) | u16::from(second & 0x3F);
            // Subtract 32 to get the actual value
            return Some((codepoint.saturating_sub(32), 2));
        }
    }

    // Invalid or unsupported encoding
    None
}

/// Parse mouse event bytes into a `MouseEvent`.
///
/// This supports both the default format and the UTF-8 multibyte format (p1005).
///
/// # Default format
/// Uses 3 raw bytes after `CSI M`:
/// - `btn`: 32 + `button_code` (with modifier bits)
/// - `col`: 32 + column (1-based)
/// - `row`: 32 + row (1-based)
///
/// # UTF-8 multibyte format (p1005)
/// Same structure but each value is encoded as a UTF-8 character:
/// - Values < 96: single byte (identical to default format)
/// - Values >= 96: 2-byte UTF-8 encoding of (value + 32) as a codepoint
/// - Range: 1 to 2015
/// - NUL byte (0x00) indicates out-of-range
///
/// # Errors
///
/// Returns an error if:
/// - The byte slice doesn't contain valid mouse data
/// - The button code is invalid
/// - Any coordinate is out of range (NUL marker)
pub fn parse_mouse_event_bytes(bytes: &[u8]) -> Result<MouseEvent, ParseError> {
    let mut offset = 0;

    // Decode button code
    let (btn_code, btn_len) =
        decode_mouse_value(&bytes[offset..]).ok_or_else(|| {
            vtansi::ParseError::InvalidValue(
                "invalid or out-of-range button code in mouse event"
                    .to_string(),
            )
        })?;
    offset += btn_len;

    // Decode column
    let (column, col_len) =
        decode_mouse_value(&bytes[offset..]).ok_or_else(|| {
            vtansi::ParseError::InvalidValue(
                "invalid or out-of-range column in mouse event".to_string(),
            )
        })?;
    offset += col_len;

    // Decode row
    let (row, _row_len) =
        decode_mouse_value(&bytes[offset..]).ok_or_else(|| {
            vtansi::ParseError::InvalidValue(
                "invalid or out-of-range row in mouse event".to_string(),
            )
        })?;

    // Default format doesn't have separate release indication, it uses button code 3
    let kind = MouseEventKind::from_button_code(btn_code, false)?;
    let modifiers = MouseKeyModifiers(modifiers_from_button_code(btn_code));

    Ok(MouseEvent {
        kind,
        modifiers,
        coords: Coordinates { column, row },
    })
}

// ============================================================================
// Default Mouse Event Encoding (CSI M btn col row - single bytes)
// ============================================================================

/// Mouse event in default reporting format.
///
/// *Sequence*: `CSI M Cb Cx Cy`
///
/// This structure encodes mouse events using the default mouse reporting
/// format, which uses `ESC [ M btn col row` with each value encoded as
/// a single byte (value + 32).
///
/// This format has a range limit of 1 to 223 for coordinates.
/// Values exceeding this range will be clamped.
///
/// This type is for encoding only - parsing is handled by
/// `parse_mouse_event_bytes` via the capture mechanism.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct DefaultMouseEvent(pub MouseEvent);

impl From<DefaultMouseEvent> for MouseEvent {
    #[inline]
    fn from(default: DefaultMouseEvent) -> Self {
        default.0
    }
}

impl From<MouseEvent> for DefaultMouseEvent {
    #[inline]
    fn from(event: MouseEvent) -> Self {
        DefaultMouseEvent(event)
    }
}

/// Encode a value in default mouse format (single byte = value + 32).
/// Values > 223 are out of range and encoded as NUL (0x00).
#[inline]
fn encode_default_mouse_value(value: u16) -> RawByte {
    // Default format: byte = value + 32, max value is 223 (byte 255)
    // Out of range values are encoded as NUL
    if value > 223 {
        RawByte(0)
    } else {
        // SAFETY: value is guaranteed to be <= 223, which fits in u8
        #[allow(clippy::cast_possible_truncation)]
        RawByte((value as u8).saturating_add(32))
    }
}

impl AnsiEncode for DefaultMouseEvent {
    fn encode_ansi_into<W: std::io::Write + ?Sized>(
        &self,
        sink: &mut W,
    ) -> Result<usize, EncodeError> {
        let event = &self.0;
        let btn_code =
            u16::from(event.kind) | u16::from(event.modifiers.bits());

        let btn_byte = encode_default_mouse_value(btn_code);
        let col_byte = encode_default_mouse_value(event.coords.column);
        let row_byte = encode_default_mouse_value(event.coords.row);

        write_csi!(sink; 'M', btn_byte, col_byte, row_byte)
    }
}

// ============================================================================
// Multibyte Mouse Event Encoding (CSI M btn col row - UTF-8 encoded)
// ============================================================================

/// Mouse event in multibyte (p1005) reporting format.
///
/// *Sequence*: `CSI M Cb Cx Cy` (UTF-8 encoded values)
///
/// This structure encodes mouse events using the UTF-8 multibyte mouse
/// reporting format, which uses `ESC [ M btn col row` with each value
/// encoded as a UTF-8 character (codepoint = value + 32).
///
/// - Values < 96: single byte (identical to default format)
/// - Values >= 96: 2-byte UTF-8 encoding
/// - Range: 1 to 2015
///
/// This type is for encoding only - parsing is handled by
/// `parse_mouse_event_bytes` via the capture mechanism.
///
/// See <https://terminalguide.namepad.de/mode/p1005/> for terminal support.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct MultibyteMouseEvent(pub MouseEvent);

impl From<MultibyteMouseEvent> for MouseEvent {
    #[inline]
    fn from(multibyte: MultibyteMouseEvent) -> Self {
        multibyte.0
    }
}

impl From<MouseEvent> for MultibyteMouseEvent {
    #[inline]
    fn from(event: MouseEvent) -> Self {
        MultibyteMouseEvent(event)
    }
}

impl AnsiEncode for MultibyteMouseEvent {
    fn encode_ansi_into<W: std::io::Write + ?Sized>(
        &self,
        sink: &mut W,
    ) -> Result<usize, EncodeError> {
        let event = &self.0;
        let btn_code =
            u16::from(event.kind) | u16::from(event.modifiers.bits());

        // Encode btn, col, row as UTF-8 codepoints
        let btn_char =
            char::from_u32(u32::from(btn_code.min(2015)) + 32).unwrap_or('\0');
        let col_char =
            char::from_u32(u32::from(event.coords.column.min(2015)) + 32)
                .unwrap_or('\0');
        let row_char =
            char::from_u32(u32::from(event.coords.row.min(2015)) + 32)
                .unwrap_or('\0');

        write_csi!(sink; 'M', btn_char, col_char, row_char)
    }
}

#[cfg(test)]
mod tests {
    use crate::event::keyboard::KeyModifiers;
    use crate::event::mouse::MouseButton;

    use super::*;

    // ========================================================================
    // SGR format tests
    // ========================================================================

    #[test]
    fn test_sgr_parse_left_click() {
        let sgr = SgrMouseEvent::try_from_ansi(b"0;10;5").unwrap();
        let event: MouseEvent = sgr.into();
        assert!(matches!(
            event.kind,
            MouseEventKind::Down(MouseButton::Left)
        ));
        assert_eq!(event.column(), 9); // 0-based
        assert_eq!(event.row(), 4); // 0-based
    }

    #[test]
    fn test_sgr_parse_right_click() {
        let sgr = SgrMouseEvent::try_from_ansi(b"2;20;15").unwrap();
        let event: MouseEvent = sgr.into();
        assert!(matches!(
            event.kind,
            MouseEventKind::Down(MouseButton::Right)
        ));
        assert_eq!(event.column(), 19);
        assert_eq!(event.row(), 14);
    }

    #[test]
    fn test_sgr_parse_scroll_up() {
        let sgr = SgrMouseEvent::try_from_ansi(b"64;10;5").unwrap();
        let event: MouseEvent = sgr.into();
        assert!(matches!(event.kind, MouseEventKind::ScrollUp));
    }

    #[test]
    fn test_sgr_mouse_to_mouse_event() {
        let sgr = SgrMouseEvent(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            modifiers: MouseKeyModifiers(KeyModifiers::NONE),
            coords: Coordinates::new(10, 20),
        });
        let event: MouseEvent = sgr.into();
        assert_eq!(event.column(), 9); // 0-based
        assert_eq!(event.row(), 19); // 0-based
    }

    // ========================================================================
    // urxvt format tests
    // ========================================================================

    #[test]
    fn test_urxvt_parse_left_click() {
        // btn=32 (0 + 32 offset), col=10, row=5
        let urxvt = UrxvtMouseEvent::try_from_ansi(b"32;10;5").unwrap();
        let event: MouseEvent = urxvt.into();
        assert!(matches!(
            event.kind,
            MouseEventKind::Down(MouseButton::Left)
        ));
        assert_eq!(event.column(), 9); // 0-based
        assert_eq!(event.row(), 4); // 0-based
    }

    #[test]
    fn test_urxvt_parse_right_click() {
        // btn=34 (2 + 32 offset), col=20, row=15
        let urxvt = UrxvtMouseEvent::try_from_ansi(b"34;20;15").unwrap();
        let event: MouseEvent = urxvt.into();
        assert!(matches!(
            event.kind,
            MouseEventKind::Down(MouseButton::Right)
        ));
        assert_eq!(event.column(), 19);
        assert_eq!(event.row(), 14);
    }

    #[test]
    fn test_urxvt_parse_scroll_up() {
        // btn=96 (64 + 32 offset), col=10, row=5
        let urxvt = UrxvtMouseEvent::try_from_ansi(b"96;10;5").unwrap();
        let event: MouseEvent = urxvt.into();
        assert!(matches!(event.kind, MouseEventKind::ScrollUp));
    }

    #[test]
    fn test_urxvt_parse_with_modifiers() {
        // btn=48 (0 + 16 ctrl + 32 offset), col=10, row=5
        let urxvt = UrxvtMouseEvent::try_from_ansi(b"48;10;5").unwrap();
        let event: MouseEvent = urxvt.into();
        assert!(event.modifiers.contains(KeyModifiers::CONTROL));
    }

    #[test]
    fn test_urxvt_parse_large_coordinates() {
        // Large coordinates that would overflow in default format
        let urxvt = UrxvtMouseEvent::try_from_ansi(b"32;500;300").unwrap();
        let event: MouseEvent = urxvt.into();
        assert_eq!(event.column(), 499);
        assert_eq!(event.row(), 299);
    }

    #[test]
    fn test_urxvt_missing_params() {
        assert!(UrxvtMouseEvent::try_from_ansi(b"32").is_err());
        assert!(UrxvtMouseEvent::try_from_ansi(b"32;10").is_err());
        assert!(UrxvtMouseEvent::try_from_ansi(b"").is_err());
    }

    // ========================================================================
    // Default/Multibyte format tests
    // ========================================================================

    #[test]
    fn test_parse_mouse_event_bytes_default_format() {
        // Left button click at column 10, row 5 (default single-byte format)
        // btn = 32 + 0 (left button) = 32 = 0x20
        // col = 32 + 10 = 42 = 0x2A
        // row = 32 + 5 = 37 = 0x25
        let event = parse_mouse_event_bytes(&[0x20, 0x2A, 0x25]).unwrap();
        assert!(matches!(
            event,
            MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                ..
            }
        ));
        assert_eq!(event.column(), 9); // 0-based
        assert_eq!(event.row(), 4); // 0-based
    }

    #[test]
    fn test_parse_mouse_event_bytes_utf8_multibyte_format() {
        // Left button click at column 100, row 50 (UTF-8 multibyte format)
        // btn = 32 + 0 = 32 = 0x20 (single byte)
        // col = 100 + 32 = 132 = U+0084 = 0xC2 0x84 (two-byte UTF-8)
        // row = 50 + 32 = 82 = 0x52 (single byte, < 128)
        let event = parse_mouse_event_bytes(&[0x20, 0xC2, 0x84, 0x52]).unwrap();
        assert!(matches!(
            event,
            MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                ..
            }
        ));
        assert_eq!(event.column(), 99); // 0-based (100 - 1)
        assert_eq!(event.row(), 49); // 0-based (50 - 1)
    }

    #[test]
    fn test_parse_mouse_event_bytes_utf8_large_coordinates() {
        // Right button click at column 200, row 150
        // btn = 32 + 2 (right button) = 34 = 0x22 (single byte)
        // col = 200 + 32 = 232 = U+00E8 = 0xC3 0xA8 (two-byte UTF-8)
        // row = 150 + 32 = 182 = U+00B6 = 0xC2 0xB6 (two-byte UTF-8)
        let event =
            parse_mouse_event_bytes(&[0x22, 0xC3, 0xA8, 0xC2, 0xB6]).unwrap();
        assert!(matches!(
            event,
            MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Right),
                ..
            }
        ));
        assert_eq!(event.column(), 199); // 0-based (200 - 1)
        assert_eq!(event.row(), 149); // 0-based (150 - 1)
    }

    #[test]
    fn test_parse_mouse_event_bytes_utf8_near_max_range() {
        // Large coordinates near max range (2015)
        // btn = 32 + 0 = 32 = 0x20
        // col = 2000 + 32 = 2032 = U+07F0 = 0xDF 0xB0 (two-byte UTF-8)
        // row = 1000 + 32 = 1032 = U+0408 = 0xD0 0x88 (two-byte UTF-8)
        let event =
            parse_mouse_event_bytes(&[0x20, 0xDF, 0xB0, 0xD0, 0x88]).unwrap();
        assert_eq!(event.column(), 1999); // 0-based (2000 - 1)
        assert_eq!(event.row(), 999); // 0-based (1000 - 1)
    }

    #[test]
    fn test_parse_mouse_event_bytes_out_of_range_nul() {
        // NUL byte (0x00) indicates out-of-range marker
        // btn = 0x00 (out of range)
        assert!(parse_mouse_event_bytes(&[0x00, 0x2A, 0x25]).is_err());
        // col = 0x00 (out of range)
        assert!(parse_mouse_event_bytes(&[0x20, 0x00, 0x25]).is_err());
        // row = 0x00 (out of range)
        assert!(parse_mouse_event_bytes(&[0x20, 0x2A, 0x00]).is_err());
    }

    #[test]
    fn test_parse_mouse_event_bytes_insufficient_bytes() {
        assert!(parse_mouse_event_bytes(&[]).is_err());
        assert!(parse_mouse_event_bytes(&[0x20]).is_err());
        assert!(parse_mouse_event_bytes(&[0x20, 0x2A]).is_err());
        // Incomplete UTF-8 sequence
        assert!(parse_mouse_event_bytes(&[0x20, 0xC2]).is_err());
    }

    // ========================================================================
    // Default format encoding tests
    // ========================================================================

    #[test]
    fn test_default_mouse_event_encode() {
        use vtansi::AnsiEncode;

        let event = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            modifiers: MouseKeyModifiers(KeyModifiers::NONE),
            coords: Coordinates::new(10, 5),
        };
        let default = DefaultMouseEvent(event);
        let encoded = default.encode_ansi().unwrap();

        // ESC [ M btn col row
        // btn = 0 + 32 = 32 = 0x20
        // col = 10 + 32 = 42 = 0x2A
        // row = 5 + 32 = 37 = 0x25
        assert_eq!(encoded, b"\x1b[M\x20\x2A\x25");
    }

    #[test]
    fn test_default_mouse_event_encode_out_of_range() {
        use vtansi::AnsiEncode;

        let event = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            modifiers: MouseKeyModifiers(KeyModifiers::NONE),
            coords: Coordinates::new(300, 200), // col exceeds 223 limit
        };
        let default = DefaultMouseEvent(event);
        let encoded = default.encode_ansi().unwrap();

        // Out of range values should be NUL (0x00)
        // btn = 0 + 32 = 32 = 0x20
        // col = 300 > 223, so NUL = 0x00
        // row = 200 + 32 = 232 = 0xE8
        assert_eq!(encoded, b"\x1b[M\x20\x00\xE8");
    }

    // ========================================================================
    // Multibyte format encoding tests
    // ========================================================================

    #[test]
    fn test_multibyte_mouse_event_encode_small_values() {
        use vtansi::AnsiEncode;

        let event = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            modifiers: MouseKeyModifiers(KeyModifiers::NONE),
            coords: Coordinates::new(10, 5),
        };
        let multibyte = MultibyteMouseEvent(event);
        let encoded = multibyte.encode_ansi().unwrap();

        // Small values (< 96) encode same as default
        assert_eq!(encoded, b"\x1b[M\x20\x2A\x25");
    }

    #[test]
    fn test_multibyte_mouse_event_encode_large_values() {
        use vtansi::AnsiEncode;

        let event = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            modifiers: MouseKeyModifiers(KeyModifiers::NONE),
            coords: Coordinates::new(100, 50),
        };
        let multibyte = MultibyteMouseEvent(event);
        let encoded = multibyte.encode_ansi().unwrap();

        // btn = 0 + 32 = 32 = 0x20 (single byte)
        // col = 100 + 32 = 132 = U+0084 = 0xC2 0x84 (two-byte UTF-8)
        // row = 50 + 32 = 82 = 0x52 (single byte, < 128)
        assert_eq!(encoded, b"\x1b[M\x20\xC2\x84\x52");
    }

    #[test]
    fn test_multibyte_mouse_event_encode_max_range() {
        use vtansi::AnsiEncode;

        let event = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            modifiers: MouseKeyModifiers(KeyModifiers::NONE),
            coords: Coordinates::new(2000, 1000),
        };
        let multibyte = MultibyteMouseEvent(event);
        let encoded = multibyte.encode_ansi().unwrap();

        // btn = 0 + 32 = 32 = 0x20
        // col = 2000 + 32 = 2032 = U+07F0 = 0xDF 0xB0
        // row = 1000 + 32 = 1032 = U+0408 = 0xD0 0x88
        assert_eq!(encoded, b"\x1b[M\x20\xDF\xB0\xD0\x88");
    }

    #[test]
    fn test_multibyte_mouse_event_roundtrip() {
        use vtansi::AnsiEncode;

        let original = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Right),
            modifiers: MouseKeyModifiers(KeyModifiers::NONE),
            coords: Coordinates::new(200, 150),
        };
        let multibyte = MultibyteMouseEvent(original);
        let encoded = multibyte.encode_ansi().unwrap();

        // Skip the "ESC [ M" prefix and parse the rest
        let parsed = parse_mouse_event_bytes(&encoded[3..]).unwrap();

        assert_eq!(parsed.kind, original.kind);
        assert_eq!(parsed.modifiers.0, original.modifiers.0);
        assert_eq!(parsed.coords, original.coords);
    }
}
